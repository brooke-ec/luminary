//! Realtime updates for global app state.
//!
//! This will be subscribed to by all clients.

// Concept adapted and inspired by: https://gist.github.com/sangelxyz/fe47e931f3536289a798eea7b5d21184

use std::convert::Infallible;

use crate::core::LuminaryEngine;
use crate::obtain;
use eyre::Result;
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
        description = "A stream of JSON patches representing updates to the global app state, in the form of Server-Sent Events",
    ))
)]
pub async fn state_events(res: &mut Response, depot: &mut Depot) {
    let engine = obtain!(depot, LuminaryEngine).clone();

    sse::stream(
        res,
        async_stream::stream! {
            let mut result = Some(serde_json::to_value(&*engine.read_list().await));
            let mut reciever = engine.channel.subscribe();
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
        },
    );
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
