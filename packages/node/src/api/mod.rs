use axum::routing::get;
use axum::Router;

pub fn router() -> Router {
    Router::new().route("/ping", get(|| async { "pong" }))
}
