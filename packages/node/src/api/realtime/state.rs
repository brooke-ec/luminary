//! Realtime updates for global app state.
//!
//! This will be subscribed to by all clients.

// Concept adapted and inspired by: https://gist.github.com/sangelxyz/fe47e931f3536289a798eea7b5d21184

use std::{convert::Infallible, sync::Arc};

use crate::core::{LuminaryEngine, LuminaryStateList};
use async_stream::stream;
use eyre::{Context, Result};
use futures_util::{Stream, StreamExt};
use log::error;
use luminary_macros::wrap_err;
use salvo::sse::SseEvent;
use serde_json::json;
use tokio::sync::{RwLock, broadcast};

#[derive(Debug)]
struct StateSnapshot {
    projects: LuminaryStateList,
    serialised: serde_json::Value,
}

/// Provides a stream of updates to the global app state.
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
    state: Arc<RwLock<StateSnapshot>>,
}

impl LuminaryStateChannel {
    /// Creates a new LuminaryLogsChannel with the given LuminaryEngine.
    #[wrap_err("Failed to create LuminaryState")]
    pub async fn setup(engine: LuminaryEngine) -> Result<Self> {
        let state = Self {
            channel: broadcast::channel(64).0,
            engine,
            state: Arc::new(RwLock::new(StateSnapshot {
                projects: LuminaryStateList::new(),
                serialised: json!({}),
            })),
        };

        state.clone().spawn_worker().await?;
        return Ok(state);
    }

    /// Spawns a background worker that listens for changes from the Luminary Engine and sends them to clients.
    #[wrap_err("Failed to spawn state worker")]
    async fn spawn_worker(self) -> Result<()> {
        self.refresh().await?;
        tokio::spawn(async move {
            let mut reciever = self.engine.stream_updates();

            while let Some(result) = reciever.next().await {
                // Flatten errors and report them
                let project = match result.wrap_err("Failed to receive project update") {
                    Ok(project) => project,
                    Err(error) => {
                        self.error(error);
                        continue;
                    }
                };

                let mut state = self.state.write().await;
                project.merge_into(&mut state.projects);

                match self.broadcast(&mut state) {
                    Ok(_) => {}
                    Err(error) => {
                        self.error(error.wrap_err("Failed to broadcast project update"));
                    }
                }
            }

            // Panic here as handling docker connection errors are out of scope
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
        let mut state = self.state.write().await;
        state.projects = self.engine.list_projects().await?;
        self.broadcast(&mut state)?;

        return Ok(());
    }

    /// Sends a [eyre::Report] to all subscribers.
    pub fn error(&self, error: eyre::Report) {
        error!("Global error sent to clients: {error:?}");
        let event = SseEvent::default()
            .name("error")
            .json(json!({
                "error": format!("{error:?}")
            }))
            .expect("Failed to serialise error event"); // This should never fail as the data is already a json value
        self.channel.send(Ok(event)).ok(); // This will only error if there are no active subscribers, so we can safely ignore it
    }

    /// Sends a diff to all subscribers and updates internal state.
    ///
    /// Should be called before updating `state.serialised` with the serialised `state.projects`.
    fn broadcast(&self, state: &mut StateSnapshot) -> Result<()> {
        let new = serde_json::to_value(&state.projects).wrap_err("Failed to serialise project list")?;
        let event = LuminaryStateChannel::generate_event(&state.serialised, &new);
        state.serialised = new;

        self.channel.send(event).ok(); // This will only error if there are no active subscribers, so we can safely ignore it
        return Ok(());
    }

    /// Generates a Server-Sent Event containing a JSON Patch diff between two states.
    fn generate_event(old: &serde_json::Value, new: &serde_json::Value) -> Result<SseEvent, Infallible> {
        let diff = json_patch::diff(&old, &new);
        let event = SseEvent::default()
            .name("patch")
            .json(diff)
            .expect("Failed to serialse JSON patch"); // This should never fail as we've already serialised the data to a json value

        return Ok(event);
    }

    /// Creates a Server-Sent Event stream for clients to subscribe to.
    pub async fn stream(&self) -> impl Stream<Item = Result<SseEvent, Infallible>> + use<> {
        // Generate an event with the old state as an empty object (as the client has no state yet)
        let event = LuminaryStateChannel::generate_event(&json!({}), &self.state.read().await.serialised);
        let mut reciever = self.channel.subscribe();

        return stream! {
            yield event; // Send the initial state as an event immediately upon connection

            loop {
                yield reciever.recv().await.expect("LuminaryStateChannel closed");
            }
        };
    }
}
