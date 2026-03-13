//! Manages real-time updates

use crate::{
    api::{auth::protected, realtime::state::state_events},
    obtain,
};
use salvo::{Depot, Response, Router, oapi::endpoint, sse};

pub use logs::LuminaryLogsChannel;

mod logs;
mod state;

/// Returns a router containing all realtime-related endpoints.
pub fn router() -> Router {
    return Router::with_path("/realtime")
        .hoop(protected)
        .push(Router::with_path("app").get(state_events))
        .push(Router::with_path("logs").get(logs_subscribe));
}

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
    let channel = obtain!(depot, LuminaryLogsChannel);
    sse::stream(res, channel.subscribe("metube".to_string()).await);
}
