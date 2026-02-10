use axum::Router;

pub fn router() -> Router {
    Router::new().route("/ping", axum::routing::get(|| async { "pong" }))
}
