//! This module implements a stream for listening to changes from Docker engine.

use std::{
    collections::HashMap,
    pin::Pin,
    task::{Context, Poll, ready},
};

use bollard::{models::EventMessage, query_parameters::EventsOptionsBuilder};
use eyre::Result;
use futures_util::{FutureExt, Stream, future::BoxFuture};
use pin_project::pin_project;

use crate::{
    LuminaryEngine,
    model::{LuminaryProjectList, LuminaryStatus},
    project::{LuminaryProjectListExt, LuminaryProjectParser},
};

impl LuminaryEngine {
    /// Subscribes to Docker events and emits an updated list of Luminary projects whenever a change occurs.
    pub fn subscribe(&self) -> impl Stream<Item = Result<LuminaryProjectList>> {
        let mut filters = HashMap::new();
        filters.insert("type", vec!["container"]);

        let options = EventsOptionsBuilder::default().filters(&filters).build();
        return LuminaryChangeStream {
            stream: self.docker.events(Some(options)),
            future: Some(self.list_projects().boxed()),
            engine: self,
            state: None,
        };
    }
}

/// A stream that listens to Docker events and emits an updated list of Luminary projects whenever a change occurs.
#[pin_project]
pub struct LuminaryChangeStream<'a, S>
where
    S: Stream<Item = Result<EventMessage, bollard::errors::Error>>,
{
    #[pin]
    stream: S,
    #[pin]
    future: Option<BoxFuture<'a, Result<LuminaryProjectList>>>,
    #[pin]
    state: Option<LuminaryProjectList>,
    engine: &'a LuminaryEngine,
}

/// Process a Docker event and update the project list accordingly.
async fn process_event<'a>(
    engine: &'a LuminaryEngine,
    mut state: LuminaryProjectList,
    event: EventMessage,
) -> Result<LuminaryProjectList> {
    if let Some(action) = event.action
        && let Some(actor) = event.actor
        && let Some(labels) = actor.attributes
        && let Some(status) = parse_action(action.clone())
        && let Some(project) = engine.parse_labels(status, labels)
    {
        state.merge_project(project);
    }

    return Ok(state);
}

/// Translate a Docker event action into a LuminaryStatus, if possible.
fn parse_action(action: String) -> Option<LuminaryStatus> {
    return match action.as_str() {
        "create" => Some(LuminaryStatus::Loading),
        "destroy" => Some(LuminaryStatus::Down),
        "start" => Some(LuminaryStatus::Running),
        "stop" => Some(LuminaryStatus::Exited),
        "restart" => Some(LuminaryStatus::Running),
        "kill" => Some(LuminaryStatus::Loading),
        "pause" => Some(LuminaryStatus::Paused),
        "unpause" => Some(LuminaryStatus::Running),
        "die" => Some(LuminaryStatus::Exited),
        "oom" => Some(LuminaryStatus::Exited),
        _ => None,
    };
}

impl<'a, S> Stream for LuminaryChangeStream<'a, S>
where
    S: Stream<Item = Result<EventMessage, bollard::errors::Error>>,
{
    type Item = Result<LuminaryProjectList>;

    // Adapted from https://docs.rs/futures-util/0.3.32/src/futures_util/stream/stream/then.rs.html#64-78
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        Poll::Ready(loop {
            // If we are currently processing an event
            if let Some(fut) = this.future.as_mut().as_pin_mut() {
                let list = ready!(fut.poll(cx))?; // Check if processing is complete - returns if not
                this.state.set(Some(list.clone())); // Update the state with the new project list
                this.future.set(None); // Clear the future to indicate we're ready for the next event
                break Some(Ok(list)); // And emit it to the subscriber
            }
            // Otherwise, poll for the next event from Docker.
            else if let Some(event) = ready!(this.stream.as_mut().poll_next(cx)) {
                let state = this.state.take().expect("State was empty for processing");
                this.future
                    .set(Some(process_event(this.engine, state, event?).boxed())); // And start processing the event
            }
            // Otherwise, Docker has closed the stream, so we will too
            else {
                break None;
            }
        })
    }
}
