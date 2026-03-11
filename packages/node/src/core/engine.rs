//! The core engine of the Luminary application, containing shared state and configuration.

use std::{path::Path, process::Stdio, sync::Arc};

use async_stream::stream;
use bollard::Docker;
use bytes::Bytes;
use eyre::{Context, Result, bail};
use futures_util::{StreamExt, stream::BoxStream};
use luminary_macros::wrap_err;
use tokio::{
    io::AsyncReadExt,
    process::Command,
    sync::{RwLock, RwLockReadGuard, broadcast},
};

use crate::core::{LuminaryStateList, configuration::LuminaryConfiguration};

/// The core engine of the Luminary application, containing shared state and configuration.
#[derive(Debug, Clone)]
pub struct LuminaryEngine {
    /// The canonical list of services for this instance of [LuminaryEngine].
    pub(super) list: Arc<RwLock<LuminaryStateList>>,

    /// A channel for broadcasting state changes to listeners.
    pub channel: broadcast::Sender<LuminaryStateList>,

    /// The configuration for Luminary Engine, loaded from environment variables.
    pub configuration: Arc<LuminaryConfiguration>,

    /// The Docker client instance for interacting with the Docker engine.
    pub(super) docker: Docker,
}

impl LuminaryEngine {
    /// Initializes a new instance of the Engine struct, loading configuration from environment variables and connecting to the Docker engine.
    #[wrap_err("Failed to create LuminaryEngine")]
    pub async fn setup() -> Result<Self> {
        let docker = Docker::connect_with_defaults().wrap_err("Failed to connect to docker engine.")?;
        let configuration = Arc::new(envy::prefixed("LUMINARY_").from_env::<LuminaryConfiguration>()?);

        let instance = Self {
            list: Arc::new(RwLock::new(LuminaryStateList::new())),
            channel: broadcast::channel(64).0,
            configuration,
            docker,
        };

        // Spawn worker task to monitor events from Docker
        instance.spawn_worker().await;
        return Ok(instance);
    }

    /// Get a immutable reference to the current list of projects and services.
    pub async fn read_list<'a>(&'a self) -> RwLockReadGuard<'a, LuminaryStateList> {
        return self.list.read().await;
    }

    /// Broadcasts the given state change to all listeners.
    pub(super) async fn broadcast(&self, list: LuminaryStateList) {
        if self.channel.receiver_count() > 0 {
            // This will only error if there are no receivers, so we can safely ignore it.
            let _ = self.channel.send(list.clone());
        }
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
