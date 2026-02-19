use axum::Router;
use color_eyre::eyre::Result;
use dotenv::dotenv;
use tokio::net::TcpListener;

mod api;
mod core;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let core = core::LuminaryCore::new()?;
    let projects = core.list_projects().await?;
    println!("Projects: {:#?}", projects);

    // let listener = TcpListener::bind("0.0.0.0:9000").await?;
    // let router = Router::new().nest("/api/", api::router());

    // println!("Listening on http://127.0.0.1:{}", listener.local_addr()?.port());
    // axum::serve(listener, router).await?;
    return Ok(());
}
