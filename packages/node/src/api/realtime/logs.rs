//! Manages real-time log streaming.

// Concept adapted and inspired by: https://gist.github.com/sangelxyz/fe47e931f3536289a798eea7b5d21184

use std::{collections::HashMap, convert::Infallible, sync::Arc};

use base64::prelude::*;
use bytes::BytesMut;
use futures_util::{Stream, StreamExt};
use salvo::sse::SseEvent;
use tokio::sync::{Mutex, RwLock, broadcast};

use crate::{api::realtime::LuminaryStateChannel, core::LuminaryEngine};

type LogChannelEntry = (
    broadcast::Sender<Result<SseEvent, Infallible>>,
    // Using an Arc here to allow the worker to keep a reference to the log buffer
    Arc<RwLock<BytesMut>>,
);

/// Manages the streaming of project logs to subscribed clients.
///
/// This channel lazily spawns a worker for each project when a a client subscribes.
/// Similarly, it terminates the worker when there are no more subscribers, avoiding unnecessary resource usage.
///
/// This is cloned for each request as children are
/// individually reference counted, making it a very cheap operation.
#[derive(Debug, Clone)]
pub struct LuminaryLogsChannel {
    engine: LuminaryEngine,
    state: LuminaryStateChannel,
    channels: Arc<
        // Using a mutex here, as unlike LuminaryStateChannel there will be multiple writers
        Mutex<HashMap<String, LogChannelEntry>>,
    >,
}

impl LuminaryLogsChannel {
    /// Creates a new LuminaryLogsChannel with the given LuminaryEngine.
    pub fn new(engine: LuminaryEngine, state: LuminaryStateChannel) -> Self {
        Self {
            engine,
            state,
            channels: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Creates a Server-Sent Event stream for clients to subscribe to.
    ///
    /// Logs are emitted as base64-encoded chunks of bytes.
    pub async fn subscribe(
        &self,
        project: String,
    ) -> impl Stream<Item = Result<SseEvent, Infallible>> + use<> {
        // Obtain entry for the project, creating a new one if neccessary
        let (sender, buffer) = self
            .channels
            .lock()
            .await
            .entry(project.clone())
            .or_insert_with(|| self.spawn_worker(project.clone()))
            .clone();

        async_stream::stream! {
            // Send previous logs in buffer to bring client up to date
            yield LuminaryLogsChannel::create_event(&buffer.read().await);
            let mut receiver = sender.subscribe();

            loop {
                match receiver.recv().await {
                    Ok(event) => yield event,
                    Err(broadcast::error::RecvError::Closed) => break,
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                }
            }
        }
    }

    /// Spawns a background worker that listens for logs from the Luminary Engine and sends them to clients.
    fn spawn_worker(&self, project: String) -> LogChannelEntry {
        let this = self.clone();

        let entry = (broadcast::channel(64).0, Arc::new(RwLock::new(BytesMut::new())));
        let (sender, buffer) = entry.clone();

        tokio::spawn(async move {
            let mut stream = this.engine.stream_logs(project.clone());

            while let Some(result) = stream.next().await {
                if let Ok(bytes) = result {
                    // Append new logs to the buffer and notify subscribers
                    buffer.write().await.extend_from_slice(&bytes);
                    if sender.send(LuminaryLogsChannel::create_event(&bytes)).is_err() {
                        break;
                    }
                } else if let Err(e) = result {
                    this.state.error(e); // Forward error to global state channel
                }
            }

            this.channels.lock().await.remove(&project);
        });

        return entry;
    }

    /// Creates a Server-Sent Event from a chunk of log bytes.
    fn create_event(bytes: &[u8]) -> Result<SseEvent, Infallible> {
        let encoded = BASE64_STANDARD.encode(bytes);
        return Ok(SseEvent::default().text(encoded));
    }
}
