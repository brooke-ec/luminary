//! Contains all the routes for Luminary Node's API.

use salvo::{Depot, Response, Router, oapi::endpoint, sse};

use crate::state::LuminaryStateChannel;

pub fn router() -> Router {
    return Router::with_path("/api")
        .push(Router::with_path("ping").get(ping))
        .push(Router::with_path("subscribe").get(subscribe));
}

/// A simple endpoint to test if the server is running.
#[endpoint]
async fn ping() -> &'static str {
    "pong"
}

/// Subscribes to a stream of updates to the global app state, including error messages and project changes.
#[endpoint(responses((
    body = String,
    status_code = 200,
    content_type = "text/event-stream",
    description = "A stream of updates to the app state in the form of Server-Sent Events",
)))]
async fn subscribe(res: &mut Response, depot: &mut Depot) {
    let channel = depot
        .obtain::<LuminaryStateChannel>()
        .expect("Depot partially populated");
    sse::stream(res, channel.stream().await);
}
