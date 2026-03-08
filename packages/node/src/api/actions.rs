//! Manages project actions

use crate::{core::LuminaryEngine, obtain, util::IntoStatusError};
use salvo::{Depot, Router, http::StatusError, oapi::endpoint};

use crate::api::auth::protected;

/// Returns a router containing all action-related endpoints.
pub fn router() -> Router {
    return Router::with_path("/action")
        .hoop(protected)
        .push(Router::with_path("restart").post(restart));
}

#[endpoint]
async fn restart(depot: &mut Depot) -> Result<(), StatusError> {
    let engine = obtain!(depot, LuminaryEngine);
    engine.restart("metube".to_string(), None).await.into_500()?;

    return Ok(());
}
