use axum::Router;
use color_eyre::eyre::Result;
use tokio::net::TcpListener;

use crate::state::LuminaryState;

mod api;
mod docker;
mod state;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:9000").await?;
    let router = Router::<LuminaryState>::new()
        .nest("/api/", api::router())
        .with_state(LuminaryState::new());

    println!("Listening on http://127.0.0.1:{}", listener.local_addr()?.port());
    axum::serve(listener, router).await?;
    return Ok(());
}
