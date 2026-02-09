use axum::{routing::get, Router};
use color_eyre::eyre::Result;
use tokio::net::TcpListener;

pub mod server;

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new().nest("/api/", Router::new().route("/ping", get(|| async { "pong" })));

    let listener = TcpListener::bind("0.0.0.0:9000").await?;
    println!("Listening on http://127.0.0.1:{}", listener.local_addr()?.port());
    axum::serve(listener, app).await?;
    return Ok(());
}
