//! The core engine of the Luminary application, containing shared state and configuration.

use std::{collections::HashMap, path::Path, process::Stdio, sync::Arc};

use async_stream::stream;
use bollard::Docker;
use bytes::Bytes;
use eyre::{Context, Result, bail};
use futures_util::{StreamExt, stream::BoxStream};
use luminary_macros::wrap_err;
use tokio::{io::AsyncReadExt, process::Command, sync::RwLock};

use crate::core::{LuminaryAction, configuration::LuminaryConfiguration};

/// The core engine of the Luminary application, containing shared state and configuration.
#[derive(Debug, Clone)]
pub struct LuminaryEngine {
    /// A map of project names to their currently processing action.
    ///
    /// Projects should be removed instead of being set to [LuminaryAction::Idle] when no action is being performed.
    pub actions: Arc<RwLock<HashMap<String, LuminaryAction>>>,
    /// The configuration for Luminary Engine, loaded from environment variables.
    pub configuration: Arc<LuminaryConfiguration>,
    /// The Docker client instance for interacting with the Docker engine.
    pub(crate) docker: Docker,
}

impl LuminaryEngine {
    /// Initializes a new instance of the Engine struct, loading configuration from environment variables and connecting to the Docker engine.
    #[wrap_err("Failed to create LuminaryEngine")]
    pub fn setup() -> Result<Self> {
        let docker = Docker::connect_with_defaults().wrap_err("Failed to connect to docker engine.")?;
        let configuration = Arc::new(envy::prefixed("LUMINARY_").from_env::<LuminaryConfiguration>()?);

        return Ok(Self {
            actions: Arc::new(RwLock::new(HashMap::new())),
            configuration,
            docker,
        });
    }

    /// Spawns a `docker compose` process and returns a stream of raw bytes merging both stdout and stderr
    #[wrap_err("Failed to spawn docker compose process")]
    pub(super) fn cli<'a>(
        &self,
        name: &'a str,
        args: impl IntoIterator<Item = &'a str>,
    ) -> Result<BoxStream<'static, Result<Bytes>>> {
        let path = Path::new(&self.configuration.project_directory).join(name);

        if !path.exists() {
            bail!("Project '{}' does not exist", name);
        }

        let mut child = Command::new("docker")
            .current_dir(path)
            .arg("compose")
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .wrap_err("Failed to spawn docker compose process")?;

        let mut stdout = child.stdout.take().expect("stdout was piped");
        let mut stderr = child.stderr.take().expect("stderr was piped");

        Ok(stream! {
            let mut stdout_buf = vec![0u8; 4096];
            let mut stderr_buf = vec![0u8; 4096];
            let mut stdout_done = false;
            let mut stderr_done = false;

            while !stdout_done || !stderr_done {
                tokio::select! {
                    result = stdout.read(&mut stdout_buf), if !stdout_done => {
                        match result {
                            Ok(0) => stdout_done = true,
                            Ok(n) => yield Ok(Bytes::copy_from_slice(&stdout_buf[..n])),
                            Err(e) => {
                                yield Err(eyre::eyre!(e).wrap_err("Failed to read stdout"));
                                stdout_done = true;
                            }
                        }
                    }
                    result = stderr.read(&mut stderr_buf), if !stderr_done => {
                        match result {
                            Ok(0) => stderr_done = true,
                            Ok(n) => yield Ok(Bytes::copy_from_slice(&stderr_buf[..n])),
                            Err(e) => {
                                yield Err(eyre::eyre!(e).wrap_err("Failed to read stderr"));
                                stderr_done = true;
                            }
                        }
                    }
                }
            }

            child.wait().await.ok();
        }
        .boxed())
    }
}
