use axum::Router;

use crate::state::LuminaryState;

pub fn router() -> Router<LuminaryState> {
    Router::new().route("/ping", axum::routing::get(|| async { "pong" }))
}
