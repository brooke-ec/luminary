use bollard::Docker;
use eyre::{Ok, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let docker = Docker::connect_with_defaults()?;

    let version = docker.version().await?;
    println!("{:?}", version);
    return Ok(());
}
