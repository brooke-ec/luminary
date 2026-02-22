use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{FutureExt, Stream, future::BoxFuture, ready};
use pin_project::pin_project;

#[pin_project]
#[must_use = "streams do nothing unless polled"]
pub struct ThenFold<'a, S, V, F, I> {
    #[pin]
    stream: S,
    #[pin]
    future: Option<BoxFuture<'a, (V, I)>>,
    state: Option<V>,
    f: F,
}

impl<'a, S, V, F, I> ThenFold<'a, S, V, F, I> {
    pub fn new(stream: S, init: V, f: F) -> Self {
        Self {
            stream,
            future: None,
            state: Some(init),
            f,
        }
    }
}

impl<'a, S, V, F, Fut, I> Stream for ThenFold<'a, S, V, F, I>
where
    Fut: Future<Output = (V, I)> + Send + 'a,
    F: Fn(V, S::Item) -> Fut,
    S: Stream<Item = V>,
{
    type Item = I;

    // Adapted from https://docs.rs/futures-util/0.3.32/src/futures_util/stream/stream/then.rs.html#64-78
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        Poll::Ready(loop {
            // If we are currently processing an item
            if let Some(fut) = this.future.as_mut().as_pin_mut() {
                let (state, item) = ready!(fut.poll(cx)); // Check if processing is complete - returns if not
                *this.state = Some(state); // Update the state with results of processing
                this.future.set(None); // Clear the future to indicate we're ready for the next event
                break Some(item); // And emit it to the subscriber
            }
            // Otherwise, poll for the next item from the stream
            else if let Some(item) = ready!(this.stream.as_mut().poll_next(cx)) {
                let state = this.state.take().unwrap();
                this.future.set(Some((this.f)(state, item).boxed())); // And start processing the item
            }
            // Otherwise, the stream has ended, so we can do the same
            else {
                break None;
            }
        })
    }
}
