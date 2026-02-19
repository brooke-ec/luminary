use std::process::Stdio;

use bollard::Docker;
use color_eyre::eyre::{Context, Result};
use luminary_macros::wrap_err;
use serde::Deserialize;
use tokio::process::Command;

mod model;
mod project;

/// A struct containing Luminary configuration, to be loaded from environment variables.
#[derive(Deserialize, Debug)]
pub struct LuminaryConfiguration {
    pub project_directory: String,
}

/// The core struct of the Luminary application, containing shared state and configuration.
#[derive(Debug)]
pub struct LuminaryCore {
    pub configuration: LuminaryConfiguration,
    docker: Docker,
}

impl LuminaryCore {
    /// Initializes a new instance of the LuminaryCore struct, loading configuration from environment variables and connecting to the Docker engine.
    pub fn new() -> Result<Self> {
        let docker = Docker::connect_with_defaults().wrap_err("Failed to connect to docker engine.")?;
        let configuration = envy::prefixed("LUMINARY_").from_env::<LuminaryConfiguration>()?;

        return Ok(Self {
            configuration,
            docker,
        });
    }

    #[wrap_err("Failed to read from docker compose command line interface")]
    async fn read_cli(&self, args: Vec<&str>) -> Result<String> {
        let output = Command::new("docker")
            .arg("compose")
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .wrap_err("Failed to spawn child process")?
            .wait_with_output()
            .await
            .wrap_err("Failed to wait on child process")?;
        let string = String::from_utf8(output.stdout).wrap_err("Invalid UTF-8 from child process")?;
        return Ok(string);
    }
}
