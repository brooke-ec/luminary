use std::sync::Arc;

use crate::{
    core::{LuminaryEngine, LuminaryStatus, ProjectLogChannel},
    eyre_fmt,
};
use bytes::{Bytes, BytesMut};
use eyre::Context;
use futures_util::{StreamExt, stream::BoxStream};
use log::{debug, error};
use tokio::sync::{RwLock, broadcast};

const EMPTY_LOGS_MESSAGE: &[u8] = b"No logs to show. Waiting for project to start...\n\r";

impl LuminaryEngine {
    /// Creates a stream of [Bytes] for clients to subscribe to.
    pub async fn logs_subscribe<'a>(&'_ self, project: String) -> BoxStream<'a, Bytes> {
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
                    yield <BytesMut as Clone>::clone(&bytes).freeze()
                } 
            }

            let mut receiver = channel.subscribe();
            debug!("Subscriber number {} subscribed to logs for project '{}'", channel.receiver_count(), project);

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
                match this
                    .cli(&project, ["logs", "-f"])
                    .wrap_err("Failed to start docker compose logs process")
                {
                    Err(err) => error!("{}", eyre_fmt!(err)),
                    Ok(mut stream) => {
                        while let Some(result) = stream.next().await {
                            match result.wrap_err("Error streaming logs for project") {
                                Err(err) => error!("{}", eyre_fmt!(err)),
                                Ok(bytes) => {
                                    let bytes = normalise_line_endings(&bytes);

                                    // Update buffer with logs and send to subscribers
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
                
                // Send a message to clients if there are no logs to show
                {
                    let mut buffer = buffer.write().await;
                    if buffer.is_empty() {
                        buffer.extend_from_slice(EMPTY_LOGS_MESSAGE);
                        let _ = channel.send(Bytes::from(EMPTY_LOGS_MESSAGE));
                    }
                }

                loop {
                    match this
                        .wait_until(&project, None, LuminaryStatus::Running)
                        .await
                        .wrap_err("Error while waiting for project to restart")
                    {
                        Err(err) => error!("{}", eyre_fmt!(err)),
                        Ok(_) => break,
                    }
                }

                debug!("Received event indicating project is running, restarting logs stream...");

                // Clear buffer to avoid sending old logs
                buffer.write().await.clear();
            }
        });

        return entry;
    }
}

/// Normalises line endings in the given bytes to be CRLF.
fn normalise_line_endings(bytes: &[u8]) -> Bytes {
    let mut out = BytesMut::with_capacity(bytes.len());

    let mut prev = 0u8;
    for &b in bytes.iter() {
        if b == b'\n' && prev != b'\r' {
            out.extend_from_slice(b"\r\n");
        } else {
            out.extend_from_slice(&[b]);
        }
        prev = b;
    }

    return out.freeze();
}
