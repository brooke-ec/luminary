//! Manages real-time log streaming.

use std::{collections::HashMap, convert::Infallible, sync::Arc};

use base64::prelude::*;
use futures_util::{Stream, StreamExt};
use log::error;
use salvo::sse::SseEvent;
use tokio::sync::{Mutex, broadcast};

use crate::core::LuminaryEngine;

/// Manages the streaming of project logs to subscribed clients.
///
/// This channel lazily spawns a worker for each project when a a client subscribes.
/// Similarly, it terminates the worker when there are no more subscribers, avoiding unnecessary resource usage.
#[derive(Debug, Clone)]
pub struct LuminaryLogsChannel {
    engine: LuminaryEngine,
    // Using a mutex here, as unlike LuminaryStateChannel there will be multiple writers
    channels: Arc<Mutex<HashMap<String, broadcast::Sender<Result<SseEvent, Infallible>>>>>,
}

impl LuminaryLogsChannel {
    /// Creates a new LuminaryLogsChannel with the given LuminaryEngine.
    pub fn new(engine: LuminaryEngine) -> Self {
        Self {
            engine,
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
        let receiver = {
            let mut channels = self.channels.lock().await;

            if let Some(sender) = channels.get(&project) {
                sender.subscribe()
            } else {
                let (sender, reciever) = broadcast::channel(64);
                channels.insert(project.clone(), sender.clone());
                self.spawn_worker(project, sender);
                reciever
            }
        };

        async_stream::stream! {
            let mut receiver = receiver;
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
    fn spawn_worker(&self, project: String, sender: broadcast::Sender<Result<SseEvent, Infallible>>) {
        let this = self.clone();

        tokio::spawn(async move {
            let mut stream = this.engine.stream_logs(project.clone());

            while let Some(result) = stream.next().await {
                match result {
                    Ok(bytes) => {
                        let encoded = BASE64_STANDARD.encode(&bytes);
                        let event = SseEvent::default().text(encoded);

                        // If there are no subscribers break, and cleanup
                        if sender.send(Ok(event)).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Error streaming logs for {project}: {e:?}");
                    }
                }
            }

            this.channels.lock().await.remove(&project);
        });
    }
}
