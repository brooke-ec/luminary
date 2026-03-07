//! The main entry point for the Luminary Node, which serves as the backend for the Luminary Panel.

use eyre::Result;
use log::debug;
use salvo::prelude::*;
use tracing_subscriber::EnvFilter;

const DATABASE: &str = "luminary.db";

mod api;
mod core;
mod util;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;

    // Set up logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Set up the app and dependencies
    let listener = TcpListener::new("0.0.0.0:9000").bind().await;
    let router = api::setup().await?;

    // Log router structure for debugging
    debug!("Router structure: {router:?}");

    // Start serving requests
    Server::new(listener).serve(router).await;
    return Ok(());
}
