use futures_util::Stream;
pub use luminary_macros::wrap_err;

mod then_fold;

impl<T: ?Sized> StreamExt for T where T: Stream {}

pub trait StreamExt: Stream {
    fn then_fold<'a, V, F, Fut, I>(self, init: V, f: F) -> then_fold::ThenFold<'a, Self, V, F, I>
    where
        Self: Stream + Sized,
        Fut: Future<Output = (V, I)> + Send + 'static,
        F: Fn(V, Self::Item) -> Fut,
    {
        then_fold::ThenFold::new(self, init, f)
    }
}
