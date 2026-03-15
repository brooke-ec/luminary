use core::fmt;

use futures_util::stream::{BoxStream, StreamExt};
use log::error;
use salvo::oapi::ToSchema;
use serde::Serialize;
use tokio::sync::broadcast;
use tracing::{
    Event, Subscriber,
    field::{Field, Visit},
};
use tracing_subscriber::{Layer, layer::Context};

/// Represents a single log message.
#[derive(Default, Serialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LogMessage {
    pub level: String,
    pub message: Option<String>,
}

impl Visit for LogMessage {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{value:?}"));
        }
    }
}

/// A custom tracing layer that broadcasts log messages to a tokio broadcast channel.
/// This is used to send log messages to the frontend in real-time.
#[derive(Clone, Debug)]
pub struct BroadcastLayer {
    channel: broadcast::Sender<LogMessage>,
}

impl BroadcastLayer {
    /// Creates a new [BroadcastLayer] with default values.
    pub fn new() -> Self {
        Self {
            channel: broadcast::channel(64).0,
        }
    }

    /// Creates a stream of [LogMessage]s for clients to read from.
    pub fn subscribe<'a>(&'_ self) -> BoxStream<'a, LogMessage> {
        let mut reciever = self.channel.subscribe();

        return async_stream::stream! {
            loop {
                match reciever.recv().await {
                    Ok(message) => yield message,
                    Err(err) => {
                        error!("Error receiving log message: {:?}", err);
                        continue;
                    }
                };
            }
        }
        .boxed();
    }
}

impl<S: Subscriber> Layer<S> for BroadcastLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = LogMessage::default();
        let metadata = event.metadata();
        visitor.level = metadata.level().to_string();
        event.record(&mut visitor);

        let _ = self.channel.send(visitor);
    }
}
