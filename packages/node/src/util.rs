use std::fmt;

use futures_util::{StreamExt, stream::BoxStream};
use log::error;
use salvo::{http::StatusError, oapi::ToSchema};
use serde::Serialize;
use tokio::sync::broadcast;
use tracing::{
    Event, Subscriber,
    field::{Field, Visit},
};
use tracing_subscriber::{Layer, layer::Context};

/// A simple macro to obtain a type from the depot. Causing a panic if the type is not present.
#[macro_export]
macro_rules! obtain {
    ($depot:expr, $type:ty) => {
        $depot.obtain::<$type>().expect(concat!(
            concat!("Tried to obtain an instance of ", stringify!($type)),
            " which the depot didn't have"
        ))
    };
}

#[macro_export]
macro_rules! get_user {
    ($depot:expr) => {
        $depot
            .get::<crate::auth::LuminaryUser>("user")
            .ok()
            .and_then(|v| Some(v))
            .expect("User can not be obtained from a unprotected endpoint.")
    };
}

pub trait IntoStatusError<T> {
    fn into_500(self) -> Result<T, StatusError>;
}

impl<T> IntoStatusError<T> for eyre::Result<T> {
    fn into_500(self) -> Result<T, StatusError> {
        self.map_err(|error| {
            error!("{error:?}");
            StatusError::internal_server_error().brief(
                serde_json::to_string(&error.chain().map(|e| e.to_string()).collect::<Vec<String>>())
                    .expect("Serialisation should not fail on a Vec<String>"),
            )
        })
    }
}

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
    pub fn new() -> Self {
        Self {
            channel: broadcast::channel(64).0,
        }
    }

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
