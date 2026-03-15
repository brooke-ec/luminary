//! Realtime updates for global app state.
//!
//! This will be subscribed to by all clients.

// Concept adapted and inspired by: https://gist.github.com/sangelxyz/fe47e931f3536289a798eea7b5d21184

use std::convert::Infallible;

use crate::obtain;
use crate::{core::LuminaryEngine, logging::BroadcastLayer};
use eyre::Result;
use futures_util::stream::select;
use futures_util::{Stream, StreamExt};
use log::error;
use salvo::{
    Depot, Response,
    oapi::endpoint,
    sse::{self, SseEvent},
};
use serde_json::json;

/// Subscribes to a stream of app events, including log messages and changes to global app state, sent as Server-Sent Events.
#[endpoint(
    security(["bearer" = []]),
    responses((
        body = String,
        status_code = 200,
        content_type = "text/event-stream",
        description = "A stream of events representing changes to the global app state, in the form of Server-Sent Events",
    ))
)]
pub async fn app_subscribe(res: &mut Response, depot: &mut Depot) {
    let engine = obtain!(depot, LuminaryEngine);
    let layer = obtain!(depot, BroadcastLayer);

    sse::stream(res, select(log_events(layer), state_events(engine).await));
}

/// A stream of [SseEvent]s representing app log messages.
pub fn log_events(layer: &BroadcastLayer) -> impl Stream<Item = Result<SseEvent, Infallible>> + use<> {
    return layer.subscribe().filter_map(|log| async move {
        let event = SseEvent::default()
            .name("log")
            .json(&log)
            .map_err(|err| {
                error!("Error serialising log message: {:?}", err);
                err
            })
            .ok()?;

        return Some(Ok(event));
    });
}

/// A stream of [SseEvent]s representing changes to global app state, in the form of JSON Patch diffs.
pub async fn state_events(
    engine: &LuminaryEngine,
) -> impl Stream<Item = Result<SseEvent, Infallible>> + use<> {
    let mut stream = engine.state_subscribe().await;
    let mut old = json!({});

    return async_stream::stream! {
        while let Some(state) = stream.next().await {
            let new = match serde_json::to_value(&state) {
                Ok(event) => event,
                Err(err) => {
                    error!("Error serialising log message: {:?}", err);
                    continue;
                }
            };

            yield generate_patch(&old, &new);
            old = new;
        }
    };
}

/// Generates a Server-Sent Event containing a JSON Patch diff between two states.
fn generate_patch(old: &serde_json::Value, new: &serde_json::Value) -> Result<SseEvent, Infallible> {
    let diff = json_patch::diff(&old, &new);
    let event = SseEvent::default()
        .name("state")
        .json(diff)
        .expect("Failed to serialse JSON patch"); // This should never fail as we've already serialised the data to a json value

    return Ok(event);
}
