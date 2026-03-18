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

    engine.restart(&project, None).await?;
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

    engine.restart(&project, Some(&service)).await?;
    return Ok(().into());
}

/// Starts the given project and all its services.
#[endpoint]
pub async fn start_project(project: PathParam<String>, depot: &mut Depot) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);

    engine.start(&project, None).await?;
    return Ok(().into());
}

/// Starts the given service of the project.
#[endpoint]
pub async fn start_service(
    project: PathParam<String>,
    service: PathParam<String>,
    depot: &mut Depot,
) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);

    engine.start(&project, Some(&service)).await?;
    return Ok(().into());
}

/// Stops the given project and all its services.
#[endpoint]
pub async fn stop_project(project: PathParam<String>, depot: &mut Depot) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);

    engine.stop(&project, None).await?;
    return Ok(().into());
}

/// Stops the given service of the project.
#[endpoint]
pub async fn stop_service(
    project: PathParam<String>,
    service: PathParam<String>,
    depot: &mut Depot,
) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);

    engine.stop(&project, Some(&service)).await?;
    return Ok(().into());
}

/// recreates the given project and all its services.
#[endpoint]
pub async fn recreate_project(project: PathParam<String>, depot: &mut Depot) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);

    engine.recreate(&project, None).await?;
    return Ok(().into());
}

/// recreates the given service of the project.
#[endpoint]
pub async fn recreate_service(
    project: PathParam<String>,
    service: PathParam<String>,
    depot: &mut Depot,
) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);

    engine.recreate(&project, Some(&service)).await?;
    return Ok(().into());
}

/// Pulls the images for all the services in the given project.
#[endpoint]
pub async fn pull_project(project: PathParam<String>, depot: &mut Depot) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);

    engine.pull(&project, None).await?;
    return Ok(().into());
}

/// Pulls the images for the given service of the project.
#[endpoint]
pub async fn pull_service(
    project: PathParam<String>,
    service: PathParam<String>,
    depot: &mut Depot,
) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);

    engine.pull(&project, Some(&service)).await?;
    return Ok(().into());
}

/// Builds the images for all the services in the given project.
#[endpoint]
pub async fn build_project(project: PathParam<String>, depot: &mut Depot) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);

    engine.build(&project, None).await?;
    return Ok(().into());
}

/// Builds the images for the given service of the project.
#[endpoint]
pub async fn build_service(
    project: PathParam<String>,
    service: PathParam<String>,
    depot: &mut Depot,
) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);

    engine.build(&project, Some(&service)).await?;
    return Ok(().into());
}
