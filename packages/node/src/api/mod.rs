//! Contains all the routes for Luminary Node's API.

use eyre::{Context, Result};
use luminary_macros::wrap_err;
use salvo::{Router, affix_state, oapi::endpoint};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

use crate::{
    DATABASE,
    api::{auth::LuminaryAuthentication, realtime::LuminaryStateChannel},
    core::LuminaryEngine,
};

mod auth;
pub mod realtime;

/// Sets up the app router and all dependencies.
#[wrap_err("Crashed while setting up")]
pub async fn setup() -> Result<Router> {
    let pool = setup_database().await?;
    let engine = LuminaryEngine::setup()?;

    // Set up the affix state with all dependencies
    let affix = affix_state::inject(LuminaryStateChannel::setup(engine.clone()).await?)
        .inject(LuminaryAuthentication::new(pool.clone()))
        .inject(engine)
        .inject(pool);

    // Set up the app router
    let router = Router::new().hoop(affix).push(
        Router::with_path("/api")
            .push(auth::router())
            .push(realtime::router())
            .push(Router::with_path("ping").get(ping)),
    );

    // Write OpenAPI documentation to file for the panel to consume
    #[cfg(debug_assertions)]
    {
        use salvo::oapi::{
            OpenApi, SecurityScheme,
            security::{Http, HttpAuthScheme},
        };

        let openapi = OpenApi::new("Luminary Node API", env!("CARGO_PKG_VERSION"))
            .add_security_scheme("bearer", SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)))
            .merge_router(&router);

        std::fs::write(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../panel/src/lib/openapi.json"),
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
        use log::info;

        info!("Populating database with debug data...");
        sqlx::query_file!("./debug.sql").execute(&pool).await?;
    }

    return Ok(pool);
}

/// A simple endpoint to test if the server is running.
#[endpoint]
async fn ping() -> &'static str {
    "pong"
}
