//! The core engine of the Luminary application, containing shared state and configuration.

use std::{process::Stdio, sync::Arc};

use bollard::Docker;
use eyre::{Context, Result};
use luminary_macros::wrap_err;
use tokio::process::Command;

use crate::configuration::LuminaryConfiguration;

/// The core engine of the Luminary application, containing shared state and configuration.
#[derive(Debug, Clone)]
pub struct LuminaryEngine {
    pub configuration: Arc<LuminaryConfiguration>,
    pub(crate) docker: Docker,
}

impl LuminaryEngine {
    /// Initializes a new instance of the Engine struct, loading configuration from environment variables and connecting to the Docker engine.
    #[wrap_err("Failed to create LuminaryEngine")]
    pub fn setup() -> Result<Self> {
        let docker = Docker::connect_with_defaults().wrap_err("Failed to connect to docker engine.")?;
        let configuration = Arc::new(envy::prefixed("LUMINARY_").from_env::<LuminaryConfiguration>()?);

        return Ok(Self {
            configuration,
            docker,
        });
    }

    #[wrap_err("Failed to read from docker compose command line interface")]
    pub(crate) async fn read_cli(&self, args: Vec<&str>) -> Result<String> {
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
