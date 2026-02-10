use axum::Router;
use color_eyre::eyre::Result;
use tokio::net::TcpListener;

mod api;
mod docker;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:9000").await?;
    println!("Listening on http://127.0.0.1:{}", listener.local_addr()?.port());
    axum::serve(listener, Router::new().nest("/api/", api::router())).await?;
    return Ok(());
}
