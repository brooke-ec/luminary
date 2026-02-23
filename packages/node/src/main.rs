//! The main entry point for the Luminary Node, which serves as the backend for the Luminary Panel.
use axum::Router;
use dotenv::dotenv;
use eyre::{Context, Result};
use luminary_macros::wrap_err;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use tokio::net::TcpListener;

use crate::state::LuminaryState;

const DATABASE: &str = "luminary.db";

mod api;
mod state;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv().ok();

    let listener = TcpListener::bind("0.0.0.0:9000").await?;
    let router = Router::<LuminaryState>::new()
        .nest("/api/", api::router())
        .with_state(setup().await?);

    println!("Listening on http://127.0.0.1:{}", listener.local_addr()?.port());
    axum::serve(listener, router).await?;
    return Ok(());
}

#[wrap_err("Crashed while setting up")]
async fn setup() -> Result<LuminaryState> {
    let options = SqliteConnectOptions::default()
        .create_if_missing(true)
        .filename(DATABASE);

    let pool = SqlitePool::connect_with(options)
        .await
        .wrap_err("Could not connect to database")?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .wrap_err("Could not migrate database")?;

    let state = LuminaryState::create(pool)?;
    state
        .clone()
        .spawn_worker()
        .await
        .wrap_err("Failed to start state worker")?;

    return Ok(state);
}
