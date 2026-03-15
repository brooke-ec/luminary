//! Manages project actions

use crate::{api::response::LuminaryResponse, core::LuminaryEngine, obtain};
use eyre::ContextCompat;
use salvo::{Depot, Request, oapi::endpoint};

/// Restarts the given project and all its services.
#[endpoint]
pub async fn restart_project(req: &mut Request, depot: &mut Depot) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);

    let project = req
        .param::<String>("project")
        .wrap_err("Expected project parameter")?;

    engine.restart(project, None).await?;
    return Ok(().into());
}

/// Restarts the given service of the project.
#[endpoint]
pub async fn restart_service(req: &mut Request, depot: &mut Depot) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);

    let service = req
        .param::<String>("service")
        .wrap_err("Expected service parameter")?;
    let project = req
        .param::<String>("project")
        .wrap_err("Expected project parameter")?;

    engine.restart(project, Some(service)).await?;
    return Ok(().into());
}
