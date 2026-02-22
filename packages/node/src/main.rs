//! The main entry point for the Luminary Node, which serves as the backend for the Luminary Panel.
use axum::Router;
use dotenv::dotenv;
use eyre::Result;
use tokio::net::TcpListener;

use crate::state::LuminaryState;

mod api;
mod state;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv().ok();

    let state = LuminaryState::create()?;
    state.clone().spawn_worker().await?;

    let listener = TcpListener::bind("0.0.0.0:9000").await?;
    let router = Router::<LuminaryState>::new()
        .nest("/api/", api::router())
        .with_state(state);

    println!("Listening on http://127.0.0.1:{}", listener.local_addr()?.port());
    axum::serve(listener, router).await?;
    return Ok(());
}
