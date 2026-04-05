//! The core engine of the Luminary application, containing shared state and configuration.

use std::{collections::HashMap, io::Read, path::Path, sync::Arc};

use async_stream::stream;
use bollard::Docker;
use bytes::Bytes;
use eyre::{Context, Result, bail};
use futures_util::{StreamExt, stream::BoxStream};
use log::error;
use luminary_macros::wrap_err;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use tokio::sync::{Mutex, RwLock, broadcast, mpsc};

use crate::{
    configuration::LuminaryConfiguration,
    core::{LuminaryProjectList, ProjectLogChannel},
};

const PTY_SIZE: PtySize = PtySize {
    rows: 40,
    cols: 80,
    pixel_width: 18,
    pixel_height: 18,
};

/// The core engine of the Luminary application, containing shared state and configuration.
#[derive(Debug, Clone)]
pub struct LuminaryEngine {
    /// The canonical list of services for this instance of [LuminaryEngine].
    pub(super) list: Arc<RwLock<LuminaryProjectList>>,

    /// A channel for broadcasting state changes to listeners.
    pub(super) list_channel: broadcast::Sender<LuminaryProjectList>,

    /// A map of log channels for each project, keyed by project name. This is lazily populated when clients subscribe to logs for a project.
    pub(super) log_channels: Arc<Mutex<HashMap<String, ProjectLogChannel>>>,

    /// The configuration for Luminary Engine, loaded from environment variables.
    pub configuration: Arc<LuminaryConfiguration>,

    /// The Docker client instance for interacting with the Docker engine.
    pub(super) docker: Docker,
}

impl LuminaryEngine {
    /// Initializes a new instance of the Engine struct, loading configuration from environment variables and connecting to the Docker engine.
    #[wrap_err("Failed to create LuminaryEngine")]
    pub async fn setup(configuration: Arc<LuminaryConfiguration>) -> Result<Self> {
        let docker = Docker::connect_with_defaults().wrap_err("Failed to connect to docker engine.")?;

        let instance = Self {
            list: Arc::new(RwLock::new(LuminaryProjectList::new())),
            log_channels: Arc::new(Mutex::new(HashMap::new())),
            list_channel: broadcast::channel(64).0,
            configuration,
            docker,
        };

        // Spawn worker task to monitor events from Docker
        instance.spawn_state_worker().await;
        instance.refresh().await?;
        return Ok(instance);
    }

    /// Broadcasts the given state change to all listeners.
    pub(super) async fn broadcast(&self, list: LuminaryProjectList) {
        if self.list_channel.receiver_count() > 0 {
            // This will only error if there are no receivers, so we can safely ignore it.
            let _ = self.list_channel.send(list.clone());
        }
    }

    /// Spawns a `docker compose` process and returns a stream of raw bytes from its output.
    pub(super) fn run_pty<'a>(
        &self,
        name: &'a str,
        args: impl IntoIterator<Item = &'a str>,
    ) -> Result<BoxStream<'static, Result<Bytes>>> {
        let path = Path::new(&self.configuration.project_directory).join(name);

        if !path.exists() {
            bail!("Project '{}' does not exist", name);
        }

        let pty = native_pty_system()
            .openpty(PTY_SIZE)
            .map_err(|e| eyre::eyre!("Failed to open PTY: {e}"))?;

        let mut command = CommandBuilder::new("docker");
        command.cwd(path);
        command.arg("compose");
        command.args(args);

        let mut child = pty
            .slave
            .spawn_command(command)
            .map_err(|err| eyre::eyre!("Failed to spawn docker compose process: {}", err))?;

        let mut reader = pty
            .master
            .try_clone_reader()
            .map_err(|err| eyre::eyre!("Failed to create PTY output reader: {}", err))?;

        // Close the slave so reader recieves EOF
        drop(pty.slave);

        let (sender, mut reciever) = mpsc::channel::<Result<Bytes>>(64);

        tokio::task::spawn_blocking(move || {
            let mut buf = vec![0u8; 4096];

            loop {
                match reader.read(&mut buf) {
                    Err(err) => error!("Error reading from PTY: {}", err),
                    Ok(0) => break,
                    Ok(n) => {
                        sender.blocking_send(Ok(Bytes::copy_from_slice(&buf[..n]))).ok();
                    }
                }
            }

            child.wait().ok();
        });

        return Ok(stream! {
            while let Some(chunk) = reciever.recv().await {
                yield chunk;
            }
        }
        .boxed());
    }
}
