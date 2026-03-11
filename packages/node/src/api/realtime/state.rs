//! Realtime updates for global app state.
//!
//! This will be subscribed to by all clients.

// Concept adapted and inspired by: https://gist.github.com/sangelxyz/fe47e931f3536289a798eea7b5d21184

use std::{convert::Infallible, sync::Arc};

use crate::core::LuminaryEngine;
use crate::obtain;
use eyre::Result;
use log::error;
use luminary_macros::wrap_err;
use salvo::{
    Depot, Response,
    oapi::endpoint,
    sse::{self, SseEvent},
};
use serde_json::json;
use tokio::sync::{RwLock, broadcast};

/// Subscribes to a stream of updates of global app state, including error messages and project changes.
#[endpoint(
    security(["bearer" = []]),
    responses((
        body = String,
        status_code = 200,
        content_type = "text/event-stream",
        description = "A stream of JSON patches representing updates to the global app state, in the form of Server-Sent Events",
    ))
)]
pub async fn state_subscribe(res: &mut Response, depot: &mut Depot) {
    let channel = obtain!(depot, LuminaryStateChannel).clone();

    sse::stream(
        res,
        async_stream::stream! {
            let mut reciever = channel.channel.subscribe();
            yield LuminaryStateChannel::generate_patch(&json!({}), &*channel.snapshot.read().await);
            core::mem::drop(channel);

            loop {
                let event = match reciever.recv().await {
                    Ok(event) => event,
                    Err(err) => {
                        error!("Error receiving state update: {:?}", err);
                        continue;
                    }
                };

                yield event;
            }
        },
    );
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

    /// A snapshot of the previously serialized state list.
    snapshot: Arc<RwLock<serde_json::Value>>,
}

impl LuminaryStateChannel {
    /// Creates a new LuminaryLogsChannel with the given LuminaryEngine.
    #[wrap_err("Failed to create LuminaryState")]
    pub async fn setup(engine: LuminaryEngine) -> Result<Self> {
        let snapshot = Arc::new(RwLock::new(serde_json::to_value(&*engine.read_list().await)?));
        let state = Self {
            channel: broadcast::channel(64).0,
            snapshot,
            engine,
        };

        state.spawn_worker();
        return Ok(state);
    }

    /// Spawns a background worker that listens for changes from the Luminary Engine and sends them to clients.
    fn spawn_worker(&self) {
        let this = self.clone();
        tokio::spawn(async move {
            let mut reciever = this.engine.channel.subscribe();

            loop {
                // Wait for the next state update and generate a patch
                let new = match reciever.recv().await.map(|state| serde_json::to_value(&state)) {
                    Ok(Ok(state)) => state,
                    _ => continue,
                };

                let mut old = this.snapshot.write().await;
                this.channel.send(Self::generate_patch(&old, &new)).ok(); // This will only error if there are no active subscribers, so we can safely ignore it
                *old = new;
            }
        });
    }

    /// Generates a Server-Sent Event containing a JSON Patch diff between two states.
    fn generate_patch(old: &serde_json::Value, new: &serde_json::Value) -> Result<SseEvent, Infallible> {
        let diff = json_patch::diff(&old, &new);
        let event = SseEvent::default()
            .name("patch")
            .json(diff)
            .expect("Failed to serialse JSON patch"); // This should never fail as we've already serialised the data to a json value

        return Ok(event);
    }

    /// Sends a [eyre::Report] to all subscribers.
    pub fn send_error(&self, error: eyre::Report) {
        error!("Global error sent to clients: {error:?}");
        let event = SseEvent::default()
            .name("error")
            .json(json!({
                "error": format!("{error:?}")
            }))
            .expect("Failed to serialise error event"); // This should never fail as the data is already a json value
        self.channel.send(Ok(event)).ok(); // This will only error if there are no active subscribers, so we can safely ignore it
    }
}
