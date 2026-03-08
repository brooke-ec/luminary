//! Manages real-time updates

use std::convert::Infallible;

use crate::{api::auth::protected, core::LuminaryEngine, obtain};
use futures_util::StreamExt;
use salvo::{
    Depot, Response, Router,
    oapi::endpoint,
    sse::{self, SseEvent},
};

pub use state::LuminaryStateChannel;

mod state;

/// Returns a router containing all realtime-related endpoints.
pub fn router() -> Router {
    return Router::with_path("/realtime")
        .hoop(protected)
        .push(Router::with_path("app").get(state_subscribe))
        .push(Router::with_path("logs").get(logs_subscribe));
}

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
    let channel = obtain!(depot, LuminaryStateChannel);
    sse::stream(res, channel.stream().await);
}

#[endpoint(
    security(["bearer" = []]),
    responses((
        body = String,
        status_code = 200,
        content_type = "text/event-stream",
        description = "A stream bytes encoding the given project's logs",
    ))
)]
pub async fn logs_subscribe(res: &mut Response, depot: &mut Depot) {
    let engine = obtain!(depot, LuminaryEngine).clone();

    sse::stream(
        res,
        async_stream::stream! {
            let mut stream = engine.logs("metube".to_string());
            while let Some(item) = stream.next().await {
                yield Ok::<SseEvent, Infallible>(SseEvent::default().text(String::from_utf8_lossy(&item.unwrap())));
            }
        },
    );
}
