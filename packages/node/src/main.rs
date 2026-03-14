//! The main entry point for the Luminary Node, which serves as the backend for the Luminary Panel.

use eyre::{Context, Result};
use log::debug;
use salvo::prelude::*;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use tracing_subscriber::{
    EnvFilter, Layer, filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt,
};

const DATABASE: &str = "luminary.db";

mod api;
mod core;
mod util;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;

    // Set up logging
    let broadcast_layer = util::BroadcastLayer::new();
    let fmt_layer = tracing_subscriber::fmt::layer().with_filter(
        EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env()?,
    );

    tracing_subscriber::registry()
        .with(broadcast_layer.clone().with_filter(LevelFilter::WARN))
        .with(fmt_layer)
        .init();

    // Set up the app and dependencies
    let listener = TcpListener::new("0.0.0.0:9000").bind().await;
    let router = api::setup(setup_database().await?, broadcast_layer).await?;

    // Log router structure for debugging
    debug!("Router structure: {router:?}");

    // Start serving requests
    Server::new(listener).serve(router).await;
    return Ok(());
}

/// Sets up the SQLite database, running any pending migrations.
async fn setup_database() -> Result<SqlitePool> {
    // Connect to the database
    let options = SqliteConnectOptions::default()
        .create_if_missing(true)
        .filename(DATABASE);

    let pool = SqlitePool::connect_with(options)
        .await
        .wrap_err("Could not connect to database")?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .wrap_err("Could not migrate database")?;

    // Populates the database with fake data for testing and development purposes.
    #[cfg(debug_assertions)]
    {
        use log::info;

        info!("Populating database with debug data...");
        sqlx::query_file!("./debug.sql").execute(&pool).await?;
    }

    return Ok(pool);
}
