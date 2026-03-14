//! Realtime updates for global app state.
//!
//! This will be subscribed to by all clients.

// Concept adapted and inspired by: https://gist.github.com/sangelxyz/fe47e931f3536289a798eea7b5d21184

use std::convert::Infallible;

use crate::obtain;
use crate::{core::LuminaryEngine, util::BroadcastLayer};
use eyre::Result;
use futures_util::Stream;
use futures_util::stream::select;
use log::error;
use salvo::{
    Depot, Response,
    oapi::endpoint,
    sse::{self, SseEvent},
};
use serde_json::json;

/// Subscribes to a stream of updates of global app state, including error messages and project changes.
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
    let engine = obtain!(depot, LuminaryEngine).clone();
    let layer = obtain!(depot, BroadcastLayer).clone();

    sse::stream(res, select(log_events(layer), state_events(engine).await));
}

pub fn log_events(layer: BroadcastLayer) -> impl Stream<Item = Result<SseEvent, Infallible>> {
    let mut reciever = layer.subscribe();

    return async_stream::stream! {
        loop {
            let result = match reciever.recv().await {
                Ok(message) => SseEvent::default().name("log").json(message),
                Err(err) => {
                    error!("Error receiving log message: {:?}", err);
                    continue;
                }
            };

            yield match result {
                Ok(event) => Ok(event),
                Err(err) => {
                    error!("Error serialising log message: {:?}", err);
                    continue;
                }
            };
        }
    };
}

pub async fn state_events(engine: LuminaryEngine) -> impl Stream<Item = Result<SseEvent, Infallible>> {
    return async_stream::stream! {
        let mut result = Some(serde_json::to_value(&*engine.read_list().await));
        let mut reciever = engine.state_channel.subscribe();
        let mut old = json!({});

        loop {
            match result.take().expect("Result should be Some at the this point") {
                Err(err) => error!("Error serialising state: {:?}", err),
                Ok(new) => {
                    yield generate_patch(&old, &new);
                    old = new;
                }
            }

            result = match reciever.recv().await {
                Ok(event) => Some(serde_json::to_value(event)),
                Err(err) => {
                    error!("Error receiving state update: {:?}", err);
                    continue;
                }
            };
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
