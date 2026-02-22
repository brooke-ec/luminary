//! Contains all the routes for Luminary Node's API.

use crate::state::LuminaryState;
use axum::Router;
use axum::extract::State;
use axum::routing::get;

pub fn router() -> Router<LuminaryState> {
    Router::new()
        .route("/ping", get(|| async { "pong" }))
        .route("/subscribe", get(subscribe))
        .nest("/projects/", Router::new())
}

async fn subscribe(state: State<LuminaryState>) -> impl axum::response::IntoResponse {
    state.channel.sse().await
}
