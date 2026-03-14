use std::sync::Arc;

use crate::core::{LuminaryEngine, LuminaryStatus, ProjectLogChannel};
use bytes::{Bytes, BytesMut};
use futures_util::{StreamExt, stream::BoxStream};
use log::{debug, error};
use tokio::sync::{RwLock, broadcast};

impl LuminaryEngine {
    /// Creates a stream of [Bytes] for clients to subscribe to.
    pub async fn subscribe<'a>(&'_ self, project: String) -> BoxStream<'a, Bytes> {
        // Obtain entry for the project, creating a new one if neccessary
        let ProjectLogChannel { channel, buffer } = self
            .log_channels
            .lock()
            .await
            .entry(project.clone())
            .or_insert_with(|| self.spawn_log_worker(project.clone()))
            .clone();

        return async_stream::stream! {
            // Surround with a block to drop read guard after reading buffer
            {
                // Send previous logs in buffer to bring client up to date
                let bytes = &buffer.read().await;
                if !bytes.is_empty() {
                    yield <BytesMut as Clone>::clone(&bytes).freeze();
                }
            }

            let mut receiver = channel.subscribe();
            loop {
                match receiver.recv().await {
                    Ok(event) => yield event,
                    Err(broadcast::error::RecvError::Closed) => break,
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                }
            }
        }
        .boxed();
    }

    /// Spawns a background worker that listens for logs sends them to clients.
    fn spawn_log_worker(&self, project: String) -> ProjectLogChannel {
        let this = self.clone();

        let entry = ProjectLogChannel {
            channel: broadcast::channel(64).0,
            buffer: Arc::new(RwLock::new(BytesMut::new())),
        };

        let ProjectLogChannel { channel, buffer } = entry.clone();
        tokio::spawn(async move {
            loop {
                debug!("Starting logs stream for project '{}'...", project);
                // Spawn docker compose process, yielding logs as they are recieved
                match this.cli(&project, ["logs", "-f"]) {
                    Err(err) => error!("Failed to start docker compose logs process: {err}"),
                    Ok(mut stream) => {
                        while let Some(result) = stream.next().await {
                            match result {
                                Err(err) => error!("Error streaming logs for project {}: {:?}", project, err),
                                Ok(bytes) => {
                                    buffer.write().await.extend_from_slice(&bytes);
                                    if channel.send(bytes).is_err() {
                                        // There are no subscribers, so clean up and stop the worker
                                        debug!("Cleaning up logs stream for project '{}'...", project);
                                        this.log_channels.lock().await.remove(&project);
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }

                // If the process exits, wait for an event from the project before triggering a retry
                debug!("Docker compose logs process exited, waiting for event to trigger retry...");

                loop {
                    match this.wait_until(&project, None, LuminaryStatus::Running).await {
                        Err(err) => error!("Error while waiting for project to restart: {err}"),
                        Ok(_) => break,
                    }
                }

                debug!("Received event indicating project is running, restarting logs stream...");
            }
        });

        return entry;
    }
}
