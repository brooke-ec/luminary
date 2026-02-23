//! The main entry point for the Luminary Node, which serves as the backend for the Luminary Panel.

use axum::Router;
use dotenv::dotenv;
use eyre::{Context, Result};
use luminary_macros::wrap_err;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use tokio::net::TcpListener;

use crate::{auth::LuminaryAuthentication, state::LuminaryState};

const DATABASE: &str = "luminary.db";

mod api;
mod auth;
mod state;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let listener = TcpListener::bind("0.0.0.0:9000").await?;
    println!("Listening on http://127.0.0.1:{}", listener.local_addr()?.port());
    axum::serve(listener, setup().await?).await?;
    return Ok(());
}

#[wrap_err("Crashed while setting up")]
async fn setup() -> Result<Router> {
    let pool = setup_database().await?;
    let state = LuminaryState::setup(pool.clone()).await?;
    let auth = LuminaryAuthentication::setup(pool).await?;

    // Set up the app router
    let router = Router::<LuminaryState>::new()
        .nest("/api/", api::router())
        .with_state(state)
        .layer(auth);

    return Ok(router);
}

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

    return Ok(pool);
}
