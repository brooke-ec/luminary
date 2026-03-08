//! Manages real-time updates

use crate::{api::auth::protected, obtain};
use salvo::{
    Depot, Response, Router, Writer,
    oapi::{endpoint, extract::QueryParam},
    sse,
};

pub use logs::LuminaryLogsChannel;
pub use state::LuminaryStateChannel;

mod logs;
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
        description = "A stream of base64-encoded log chunks for the given project, in the form of Server-Sent Events",
    ))
)]
pub async fn logs_subscribe(project: QueryParam<String, true>, res: &mut Response, depot: &mut Depot) {
    let channel = obtain!(depot, LuminaryLogsChannel);
    sse::stream(res, channel.subscribe(project.into_inner()).await);
}
