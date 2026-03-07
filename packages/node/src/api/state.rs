//! The main state management for the Luminary Node

use std::{convert::Infallible, sync::Arc};

use crate::core::{LuminaryEngine, LuminaryProject, LuminaryProjectList};
use async_stream::stream;
use eyre::{Context, Result};
use futures_util::{Stream, StreamExt};
use log::error;
use luminary_macros::wrap_err;
use salvo::{
    Depot, Response,
    oapi::endpoint,
    sse::{self, SseEvent},
};
use serde_json::json;
use tokio::sync::{RwLock, RwLockWriteGuard, broadcast};

/// Subscribes to a stream of updates to the global app state, including error messages and project changes.
#[endpoint(
    security(["bearer" = []]),
    responses((
        body = String,
        status_code = 200,
        content_type = "text/event-stream",
        description = "A stream of updates to the app state in the form of Server-Sent Events",
    ))
)]
pub async fn subscribe(res: &mut Response, depot: &mut Depot) {
    let channel = depot
        .obtain::<LuminaryStateChannel>()
        .expect("Depot partially populated");
    sse::stream(res, channel.stream().await);
}

/// Shared state for the Luminary Node.
///
/// This is cloned for each request as children are
/// individually reference counted, making it a very cheap operation.
#[derive(Debug, Clone)]
pub struct LuminaryStateChannel {
    /// The Luminary Engine, which manages containers and their state.
    engine: LuminaryEngine,

    /// A channel for sending state updates to users.
    channel: broadcast::Sender<Result<SseEvent, Infallible>>,

    /// The internal state of the channel, used to generate diffs for updates.
    state: Arc<RwLock<(LuminaryProjectList, serde_json::Value)>>,
}

impl LuminaryStateChannel {
    /// Creates a new LuminaryState with default values, and starts the state worker.
    #[wrap_err("Failed to create LuminaryState")]
    pub async fn setup(engine: LuminaryEngine) -> Result<Self> {
        let state = Self {
            state: Arc::new(RwLock::new((LuminaryProjectList::new(), json!({})))),
            channel: broadcast::channel(64).0,
            engine,
        };

        state.clone().spawn_worker().await?;
        return Ok(state);
    }

    /// Spawns a background worker that listens for changes from the Luminary Engine and sends them to clients.
    #[wrap_err("Failed to spawn state worker")]
    async fn spawn_worker(self) -> Result<()> {
        self.refresh().await?;
        tokio::spawn(async move {
            let mut reciever = self.engine.stream();

            while let Some(result) = reciever.next().await {
                // Flatten errors and report them
                if let Err(e) = self.worker_iteration(result).await {
                    self.error(e);
                }
            }

            // This should only happen if the user is meddling with stuff
            error!("Connection to Docker Engine lost, panicing");
            panic!();
        });

        return Ok(());
    }

    /// Synchronises [LuminaryStateChannel]'s internal state with the current state of the Luminary Engine.
    ///
    /// This should only need to be called, when needing to update from disk.
    /// This is because [LuminaryEngine::stream] will theoretically emit all neccesary updates to keep the state in sync.
    #[wrap_err("Failed to refresh LuminaryState")]
    pub async fn refresh(&self) -> Result<()> {
        let list = self.engine.list_projects().await?;
        let mut guard = self.state.write().await;
        *guard = (list, json!({}));
        self.send_changes(guard)?;

        return Ok(());
    }

    /// Sends a [eyre::Report] to all subscribers.
    fn error(&self, error: eyre::Report) {
        error!("Global error sent to clients: {error:?}");
        let event = SseEvent::default()
            .name("error")
            .json(json!({
                "error": format!("{error:?}")
            }))
            .expect("Failed to serialise error event"); // This should never fail as the data is already a json value
        self.channel.send(Ok(event)).ok(); // This will only error if there are no active subscribers, so we can safely ignore it
    }

    /// Handles a single iteration of the worker loop, merging changes into the internal state and sending diffs to subscribers.
    async fn worker_iteration(&self, result: Result<LuminaryProject>) -> Result<()> {
        let project = result?;
        let mut guard = self.state.write().await;
        project.merge_into(&mut guard.0);
        self.send_changes(guard)?;
        Ok(())
    }

    /// Sends a diff to all subscribers and updates internal state.
    ///
    /// Should be called before updating `state.1` with the serialised `state.0`.
    fn send_changes(
        &self,
        mut guard: RwLockWriteGuard<'_, (LuminaryProjectList, serde_json::Value)>,
    ) -> Result<()> {
        let current = serde_json::to_value(&guard.0).wrap_err("Failed to serialise project list")?;
        let event = LuminaryStateChannel::generate_event(&guard.1, &current);
        guard.1 = current;

        self.channel.send(event).ok(); // This will only error if there are no active subscribers, so we can safely ignore it
        return Ok(());
    }

    /// Generates a Server-Sent Event containing a JSON Patch diff between two states.
    fn generate_event(left: &serde_json::Value, right: &serde_json::Value) -> Result<SseEvent, Infallible> {
        let diff = json_patch::diff(&left, &right);
        let event = SseEvent::default()
            .name("patch")
            .json(diff)
            .expect("Failed to serialse JSON patch"); // This should never fail as we've already serialised the data to a json value

        return Ok(event);
    }

    /// Creates a Server-Sent Event stream for clients to subscribe to.
    pub async fn stream(&self) -> impl Stream<Item = Result<SseEvent, Infallible>> + use<> {
        let event = LuminaryStateChannel::generate_event(&json!({}), &self.state.read().await.1);
        let mut reciever = self.channel.subscribe();

        return stream! {
            yield event; // Send the initial state as an event immediately upon connection

            loop {
                yield reciever.recv().await.expect("LuminaryStateChannel closed");
            }
        };
    }
}
