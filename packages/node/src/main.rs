//! The main entry point for the Luminary Node, which serves as the backend for the Luminary Panel.

use axum::Router;
use dotenv::dotenv;
use eyre::Result;
use tokio::net::TcpListener;

mod api;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let engine = luminary_core::LuminaryEngine::default()?;
    let projects = engine.list_projects().await?;
    println!("Projects: {:#?}", projects);

    // let listener = TcpListener::bind("0.0.0.0:9000").await?;
    // let router = Router::new().nest("/api/", api::router());

    // println!("Listening on http://127.0.0.1:{}", listener.local_addr()?.port());
    // axum::serve(listener, router).await?;
    return Ok(());
}
