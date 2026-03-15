//! Manages project actions

use crate::{api::response::LuminaryResponse, core::LuminaryEngine, obtain};
use salvo::{Depot, Router, oapi::endpoint};

use crate::api::auth::protected;

/// Returns a router containing all action-related endpoints.
pub fn router() -> Router {
    return Router::with_path("/action")
        .hoop(protected)
        .push(Router::with_path("restart").post(restart));
}

#[endpoint]
async fn restart(depot: &mut Depot) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);
    engine.restart("metube".to_string(), None).await?;
    return Ok(().into());
}
