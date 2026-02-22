use std::convert::Infallible;

use axum::response::Sse;
use axum::routing::get;
use axum::{Router, response::sse::Event};
use futures_util::{Stream, StreamExt};

use crate::ENGINE;

pub fn router() -> Router {
    Router::new()
        .route("/ping", get(|| async { "pong" }))
        .route("/containers", get(containers))
}

async fn containers() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    return Sse::new(
        ENGINE
            .get()
            .unwrap()
            .subscribe()
            .map(|r| Ok(Event::default().data(serde_json::to_string(&r.unwrap()).unwrap()))),
    );
}
