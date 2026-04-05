use crate::{
    core::{LuminaryEngine, LuminaryStatus, ProjectLogChannel, ProjectLogChannelMessage},
    eyre_fmt,
};
use bytes::{Bytes, BytesMut};
use eyre::Context;
use futures_util::{StreamExt, stream::BoxStream};
use log::{debug, error};
use tokio::sync::{broadcast};
use uuid::Uuid;

const EMPTY_LOGS_MESSAGE: &[u8] = b"No logs to show. Waiting for project to start...\n\r";
const LOG_WORKER_STREAM_UUID: Uuid = Uuid::from_u128(0x0);

impl LuminaryEngine {
    /// Creates a stream of [Bytes] for clients to subscribe to.
    pub async fn logs_subscribe<'a>(self, project: String) -> BoxStream<'a, ProjectLogChannelMessage> {
        let ProjectLogChannel { channel, state } = self.get_log_channel(&project).await;

        return async_stream::stream! {
            // Surround with a block to drop read guard after reading buffer
            {                
                // Spawn log worker if it hasn't been spawned already.
                // Use temporary read guard as spawn_log_worker needs a write guard
                if !state.read().await.contains_key(&LOG_WORKER_STREAM_UUID) {
                    self.spawn_log_worker(project.clone()).await;
                }
                
                // Send previous logs in buffer to bring client up to date
                for (uuid, buffer) in state.read().await.iter() {
                    if !buffer.is_empty() {
                        yield ProjectLogChannelMessage::Write(*uuid, <BytesMut as Clone>::clone(&buffer).freeze());
                    } 
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

    /// Creates a new log sender for the given project. This will send a close message when dropped.
    pub async fn create_log_sender(&self, project: String) -> ProjectLogChannelSender {
        let channel = self.get_log_channel(&project).await;

        ProjectLogChannelSender {
            uuid: Uuid::new_v4(),
            engine: self.clone(),
            project: project,
            channel,
        }
    }

    /// Obtain the [ProjectLogChannel] for the given project, creating a new one if neccessary
    async fn get_log_channel(&self, project: &String) -> ProjectLogChannel {
        return self
            .log_channels
            .lock()
            .await
            .entry(project.clone())
            .or_default()
            .clone();
    }

    /// Spawns a background worker that listens for logs sends them to clients.
    async fn spawn_log_worker(&self, project: String) {
        let this = self.clone();
        
        let channel = self.get_log_channel(&project).await;
        channel.state.write().await.insert(LOG_WORKER_STREAM_UUID, BytesMut::new());

        let mut sender = this.create_log_sender(project.clone()).await;
        sender.uuid = LOG_WORKER_STREAM_UUID; // Use reserved UUID for log worker stream

        tokio::spawn(async move {

            loop {
                debug!("Starting logs stream for project '{}'...", project);
                // Spawn docker compose process, yielding logs as they are recieved
                match this
                    .run_pty(&project, ["logs", "-f"])
                    .wrap_err("Failed to start docker compose logs process")
                {
                    Err(err) => error!("{}", eyre_fmt!(err)),
                    Ok(mut stream) => {
                        while let Some(result) = stream.next().await {
                            match result.wrap_err("Error streaming logs for project") {
                                Err(err) => error!("{}", eyre_fmt!(err)),
                                Ok(bytes) => {
                                    if sender.write(bytes).await {
                                        sender.close().await;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }

                // If the process exits, wait for an event from the project before triggering a retry
                debug!("Docker compose logs process exited, waiting for event to trigger retry...");
                
                // Send a message to clients if there are no logs to show
                if sender.is_empty().await {
                    sender.write(Bytes::from(EMPTY_LOGS_MESSAGE)).await;
                }

                loop {
                    match this
                        .wait_until_status(&project, None, LuminaryStatus::Running)
                        .await
                        .wrap_err("Error while waiting for project to restart")
                    {
                        Err(err) => error!("{}", eyre_fmt!(err)),
                        Ok(_) => break,
                    }
                }

                debug!("Received event indicating project is running, restarting logs stream...");

                // Clear buffer to avoid sending old logs
                sender.clear().await;
            }
        });
    }
}

/// A wrapper representing a given project log stream, to be multiplexed and sent to clients.
#[derive(Debug)]
pub struct ProjectLogChannelSender {
    channel: ProjectLogChannel,
    engine: LuminaryEngine,
    project: String,
    uuid: Uuid,
}

impl ProjectLogChannelSender {
    //// Sends logs to clients and updates internal buffer. Returns true if there are no subscribers to receive the logs.
    pub async fn write(&self, bytes: Bytes) -> bool {
        let bytes = normalise_line_endings(&bytes);

        // Update buffer with new bytes
        self.channel.state.write().await.entry(self.uuid).or_default().extend_from_slice(&bytes);

        // Send bytes to clients, and record if there are no subscribers
        return self.channel.channel.send(ProjectLogChannelMessage::Write(self.uuid, bytes)).is_err();
    }

    /// Clears the internal buffer for this log stream.
    pub async fn clear(&self) {
        self.channel.state.write().await.entry(self.uuid).or_default().clear();
    }

    /// Checks if the internal buffer for this log stream is empty.
    pub async fn is_empty(&self) -> bool {
        return self.channel.state.read().await.get(&self.uuid).map(|buffer| buffer.is_empty()).unwrap_or(true);
    }
    
    /// Closes this log sender, removing its buffer and notifying clients.
    pub async fn close(&self) {
        debug!("Closing log sender '{}' for project '{}'", &self.uuid, &self.project);
        let mut streams = self.channel.state.write().await;
        streams.remove(&self.uuid);

        if self.channel.channel.send(ProjectLogChannelMessage::Close(self.uuid)).is_err() {
            if streams.is_empty() {
                debug!("No more log streams for project '{}', removing log channel...", &self.project);
                self.engine.log_channels.lock().await.remove(&self.project);
            }
        } else if streams.is_empty() {
            error!("Log channel for '{}' has no streams but still has subscribers, report this!", &self.project);
        }
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
