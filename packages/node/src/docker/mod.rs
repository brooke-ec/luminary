use std::process::Stdio;

use color_eyre::eyre::{Context, Result};
use luminary_macros::wrap_err;
use tokio::process::Command;

mod list;

pub struct DockerClient {}

impl DockerClient {
    pub fn new() -> Self {
        Self {}
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
