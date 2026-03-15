//! Manages project actions

use crate::{api::response::LuminaryResponse, core::LuminaryEngine, obtain};
use salvo::{Depot, Request, Router, oapi::endpoint};

/// Returns a router containing all action-related endpoints.
pub fn router() -> Router {
    return Router::new().push(Router::with_path("restart").post(restart));
}

#[endpoint]
async fn restart(req: &mut Request, depot: &mut Depot) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);

    let service = req.param::<String>("service");
    let project = req
        .param::<String>("project")
        .expect("Expected project parameter");

    engine.restart(project, service).await?;
    return Ok(().into());
}
