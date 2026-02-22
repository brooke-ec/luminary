//! The main entry point for the Luminary Node, which serves as the backend for the Luminary Panel.

use std::sync::OnceLock;

use axum::Router;
use dotenv::dotenv;
use eyre::{Context, Result};
use luminary_core::LuminaryEngine;
use tokio::net::TcpListener;

mod api;

static ENGINE: OnceLock<LuminaryEngine> = OnceLock::new();

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let engine = LuminaryEngine::create().wrap_err("Error creating Luminary Engine")?;
    ENGINE.set(engine).expect("Somehow, ENGINE was already set");

    let listener = TcpListener::bind("0.0.0.0:9000").await?;
    let router = Router::new().nest("/api/", api::router());

    println!("Listening on http://127.0.0.1:{}", listener.local_addr()?.port());
    axum::serve(listener, router).await?;
    return Ok(());
}
