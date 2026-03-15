//! Manages project actions

use crate::{api::response::LuminaryResponse, core::LuminaryEngine, obtain};
use salvo::{
    Depot, Writer,
    oapi::{endpoint, extract::PathParam},
};

/// Restarts the given project and all its services.
#[endpoint]
pub async fn restart_project(project: PathParam<String>, depot: &mut Depot) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);

    engine.restart(project.into_inner(), None).await?;
    return Ok(().into());
}

/// Restarts the given service of the project.
#[endpoint]
pub async fn restart_service(
    project: PathParam<String>,
    service: PathParam<String>,
    depot: &mut Depot,
) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);

    engine
        .restart(project.into_inner(), Some(service.into_inner()))
        .await?;
    return Ok(().into());
}
