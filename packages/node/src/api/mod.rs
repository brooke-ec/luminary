//! Contains all the routes for Luminary Node's API.

use crate::state::LuminaryState;
use axum::Router;
use axum::routing::get;

pub mod state;

pub fn router() -> Router<LuminaryState> {
    Router::new()
        .route("/ping", get(|| async { "pong" }))
        .route("/containers", get(state::handle_request))
}
