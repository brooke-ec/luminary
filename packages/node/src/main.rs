//! The main entry point for the Luminary Node, which serves as the backend for the Luminary Panel.

use eyre::{Context, Result};
use log::{debug, info};
use luminary_core::LuminaryEngine;
use luminary_macros::wrap_err;
use salvo::oapi::SecurityScheme;
use salvo::oapi::security::{Http, HttpAuthScheme};
use salvo::prelude::*;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use tracing_subscriber::EnvFilter;

use crate::{auth::LuminaryAuthentication, state::LuminaryStateChannel};

const DATABASE: &str = "luminary.db";

mod api;
mod auth;
mod state;
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
    let router = setup().await?;

    // Log router structure for debugging
    debug!("Router structure: {router:?}");

    // Start serving requests
    Server::new(listener).serve(router).await;
    return Ok(());
}

/// Sets up the app router and all dependencies.
#[wrap_err("Crashed while setting up")]
async fn setup() -> Result<Router> {
    let pool = setup_database().await?;
    let engine = LuminaryEngine::setup()?;

    // Set up the affix state with all dependencies
    let affix = affix_state::inject(LuminaryStateChannel::setup(engine.clone()).await?)
        .inject(LuminaryAuthentication::new(pool.clone()))
        .inject(engine)
        .inject(pool);

    // Set up the app router
    let router = Router::new().hoop(affix).push(api::router());

    // Write OpenAPI documentation to file for the panel to consume
    #[cfg(debug_assertions)]
    {
        let openapi = OpenApi::new("Luminary Node API", env!("CARGO_PKG_VERSION"))
            .add_security_scheme("bearer", SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)))
            .merge_router(&router);

        std::fs::write(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../panel/static/openapi.json"),
            openapi.to_pretty_json()?,
        )?;
    }

    return Ok(router);
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
        info!("Populating database with debug data...");
        sqlx::query_file!("./debug.sql").execute(&pool).await?;
    }

    return Ok(pool);
}
