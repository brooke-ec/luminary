use axum::{routing::get, Router};
use eyre::Result;
use tokio::net::TcpListener;

pub mod server;

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new().nest(
        "/api/",
        Router::new().route("/ping", get(|| async { "pong" })),
    );

    let listener = TcpListener::bind("0.0.0.0:9000").await?;
    axum::serve(listener, app).await?;
    return Ok(());
}
