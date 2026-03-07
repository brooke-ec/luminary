//! Manages real-time updates

use crate::api::auth::protected;
use salvo::{Depot, Response, Router, oapi::endpoint, sse};

pub use state::LuminaryStateChannel;

mod state;

/// Returns a router containing all realtime-related endpoints.
pub fn router() -> Router {
    return Router::with_path("/realtime")
        .hoop(protected)
        .push(Router::with_path("app").post(subscribe));
}

/// Subscribes to a stream of updates of global app state, including error messages and project changes.
#[endpoint(
    security(["bearer" = []]),
    responses((
        body = String,
        status_code = 200,
        content_type = "text/event-stream",
        description = "A stream of updates to the app state in the form of Server-Sent Events",
    ))
)]
pub async fn subscribe(res: &mut Response, depot: &mut Depot) {
    let channel = depot
        .obtain::<LuminaryStateChannel>()
        .expect("Depot partially populated");
    sse::stream(res, channel.stream().await);
}
