//! Manages real-time log streaming.

// Concept adapted and inspired by: https://gist.github.com/sangelxyz/fe47e931f3536289a798eea7b5d21184

use std::convert::Infallible;

use base64::prelude::*;
use futures_util::StreamExt;
use salvo::{
    Depot, Response,
    oapi::endpoint,
    sse::{self, SseEvent},
};

use crate::{core::LuminaryEngine, obtain};

#[endpoint(
    security(["bearer" = []]),
    responses((
        body = String,
        status_code = 200,
        content_type = "text/event-stream",
        description = "A stream of base64-encoded log chunks for the given project, in the form of Server-Sent Events",
    ))
)]
pub async fn logs_subscribe(res: &mut Response, depot: &mut Depot) {
    let engine = obtain!(depot, LuminaryEngine);
    let mut stream = engine.subscribe("metube".to_string()).await;

    sse::stream(
        res,
        async_stream::stream! {
            while let Some(bytes) = stream.next().await {
                match create_event(&bytes) {
                    Err(err) =>log::error!("Failed to create SSE event from log bytes: {err}"),
                    Ok(event) => yield Ok::<SseEvent, Infallible>(event),
                }
            }
        },
    );
}

/// Creates a Server-Sent Event from a chunk of log bytes.
fn create_event(bytes: &[u8]) -> Result<SseEvent, Infallible> {
    let encoded = BASE64_STANDARD.encode(bytes);
    return Ok(SseEvent::default().text(encoded));
}
