use std::{
    collections::HashMap,
    pin::Pin,
    task::{Context, Poll, ready},
};

use bollard::{models::EventMessage, query_parameters::EventsOptionsBuilder};
use eyre::Result;
use futures_util::{FutureExt, Stream, future::BoxFuture};
use pin_project::pin_project;

use crate::{LuminaryEngine, model::LuminaryProjectList};

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

async fn process_event<'a>(
    _engine: &'a LuminaryEngine,
    state: LuminaryProjectList,
    _event: EventMessage,
) -> Result<LuminaryProjectList> {
    return Ok(state); // TODO: Implement actual event processing logic to update the project list based on the event.
}

impl<'a, S> Stream for LuminaryChangeStream<'a, S>
where
    S: Stream<Item = Result<EventMessage, bollard::errors::Error>>,
{
    type Item = Result<LuminaryProjectList>;

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
