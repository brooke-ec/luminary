use axum::extract::FromRef;
use eyre::Result;
use luminary_core::{LuminaryEngine, LuminaryProject};
use luminary_macros::wrap_err;
use tokio::sync::broadcast;

/// Shared state for the Luminary Node.
///
/// This is cloned for each request as children are
/// individually reference counted, making it a very cheap operation.
#[derive(Debug, Clone)]
pub struct LuminaryState {
    /// A channel of container updates used for global events.
    pub channel: broadcast::Sender<LuminaryProject>,

    /// The Luminary Engine, which manages containers and their state.
    pub engine: LuminaryEngine,
}

impl LuminaryState {
    #[wrap_err("Failed to create LuminaryState")]
    pub fn create() -> Result<Self> {
        Ok(Self {
            channel: broadcast::channel(64).0,
            engine: LuminaryEngine::create()?,
        })
    }
}

impl FromRef<LuminaryState> for LuminaryEngine {
    fn from_ref(state: &LuminaryState) -> Self {
        return state.engine.clone();
    }
}
