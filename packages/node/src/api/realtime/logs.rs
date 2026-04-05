//! Manages real-time log streaming.

// Concept adapted and inspired by: https://gist.github.com/sangelxyz/fe47e931f3536289a798eea7b5d21184

use std::convert::Infallible;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use eyre::Context;
use futures_util::StreamExt;
use salvo::{
    Depot, Response, Writer,
    oapi::{endpoint, extract::PathParam},
    sse::{self, SseEvent},
};

use crate::{
    core::{LuminaryEngine, ProjectLogChannelMessage},
    eyre_fmt, obtain,
};

/// Subscribes to a stream of log messages for a given project, sent as Server-Sent Events.
#[endpoint(
    security(["bearer" = []]),
    responses((
        body = String,
        status_code = 200,
        content_type = "text/event-stream",
        description = "A stream of base64-encoded log chunks for the given project, in the form of Server-Sent Events",
    ))
)]
pub async fn logs_subscribe(project: PathParam<String>, res: &mut Response, depot: &mut Depot) {
    let engine = obtain!(depot, LuminaryEngine);
    let project = project.into_inner();

    let mut stream = engine.clone().logs_subscribe(project).await;

    sse::stream(
        res,
        async_stream::stream! {
            while let Some(message) = stream.next().await {
                match create_event(message).wrap_err("Failed to create SSE event from log bytes") {
                    Err(err) => log::error!("{}", eyre_fmt!(err)),
                    Ok(event) => yield Ok::<SseEvent, Infallible>(event),
                }
            }
        },
    );
}

/// Creates a Server-Sent Event from a chunk of log bytes.
fn create_event(message: ProjectLogChannelMessage) -> Result<SseEvent, Infallible> {
    return match message {
        ProjectLogChannelMessage::Close(uuid) => Ok(SseEvent::default().name("close").text(uuid)),
        ProjectLogChannelMessage::Write(uuid, bytes) => {
            let encoded = STANDARD.encode(bytes);
            Ok(SseEvent::default().name(uuid).text(encoded))
        }
    };
}
