use std::{convert::Infallible, sync::Arc};

use async_stream::stream;
use axum::{
    extract::FromRef,
    response::{Sse, sse::Event},
};
use eyre::{Context, Result};
use futures_util::{Stream, StreamExt};
use luminary_core::{LuminaryEngine, LuminaryProject, LuminaryProjectList};
use luminary_macros::wrap_err;
use serde_json::json;
use tokio::sync::{RwLock, RwLockWriteGuard, broadcast};

/// Shared state for the Luminary Node.
///
/// This is cloned for each request as children are
/// individually reference counted, making it a very cheap operation.
#[derive(Debug, Clone)]
pub struct LuminaryState {
    /// A channel of container updates used for global events.
    pub channel: LuminaryStateChannel,

    /// The Luminary Engine, which manages containers and their state.
    pub engine: LuminaryEngine,
}

impl LuminaryState {
    /// Creates a new LuminaryState with default values.
    #[wrap_err("Failed to create LuminaryState")]
    pub fn create() -> Result<Self> {
        Ok(Self {
            engine: LuminaryEngine::create()?,
            channel: LuminaryStateChannel::new(),
        })
    }

    /// Spawns a background worker that listens for changes from the Luminary Engine and sends them to clients.
    #[wrap_err("Failed to spawn LuminaryState worker")]
    pub async fn spawn_worker(self) -> Result<()> {
        self.refresh().await?;
        tokio::spawn(async move {
            let mut reciever = self.engine.stream();

            while let Some(result) = reciever.next().await {
                // Flatten errors and report them
                if let Err(e) = self.channel.worker_iteration(result).await {
                    self.channel.error(e);
                }
            }

            // This should only happen if the user is meddling with stuff
            panic!("Connection to Docker Engine lost");
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
        let mut guard = self.channel.state.write().await;
        *guard = (list, json!({}));
        self.channel.send_changes(guard)?;

        return Ok(());
    }
}

impl FromRef<LuminaryState> for LuminaryEngine {
    fn from_ref(state: &LuminaryState) -> Self {
        return state.engine.clone();
    }
}

/// Represents the main channel for sending updates to the frontend.
///
/// This is implemented with a broadcast channel, which allows for multiple subscribers to receive updates without needing to manage individual connections.
/// This channel sends diffs, which are generated using an internal state to track how each emission changes the document.
#[derive(Debug, Clone)]
pub struct LuminaryStateChannel {
    channel: broadcast::Sender<Result<Event, Infallible>>,
    state: Arc<RwLock<(LuminaryProjectList, serde_json::Value)>>,
}

impl LuminaryStateChannel {
    /// Creates a new LuminaryStateChannel with default values.
    ///
    /// This will initialise the channel with an empty project list, use [LuminaryState::refresh]
    /// or [LuminaryState::spawn_worker] to update it with the current state of the engine.
    fn new() -> Self {
        return Self {
            channel: broadcast::channel(64).0,
            state: Arc::new(RwLock::new((LuminaryProjectList::new(), json!({})))),
        };
    }

    /// Sends a [eyre::Report] to all subscribers.
    fn error(&self, error: eyre::Report) {
        println!("{}", error);
        let event = Event::default()
            .event("error")
            .json_data(json!({
                "error": format!("{error:#}")
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
    fn generate_event(left: &serde_json::Value, right: &serde_json::Value) -> Result<Event, Infallible> {
        let diff = json_patch::diff(&left, &right);
        let event = Event::default()
            .event("patch")
            .json_data(diff)
            .expect("Failed to serialse JSON patch"); // This should never fail as we've already serialised the data to a json value

        return Ok(event);
    }

    /// Creates a Server-Sent Event stream for clients to subscribe to.
    pub async fn sse(&self) -> Sse<impl Stream<Item = Result<Event, Infallible>> + use<>> {
        let event = LuminaryStateChannel::generate_event(&json!({}), &self.state.read().await.1);
        let mut reciever = self.channel.subscribe();

        return Sse::new(stream! {
            yield event; // Send the initial state as an event immediately upon connection

            loop {
                yield reciever.recv().await.expect("LuminaryStateChannel closed");
            }
        });
    }
}
