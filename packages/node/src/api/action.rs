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

    engine.restart(&project.into_inner(), None).await?;
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
        .restart(&project.into_inner(), Some(&service.into_inner()))
        .await?;
    return Ok(().into());
}

/// Starts the given project and all its services.
#[endpoint]
pub async fn start_project(project: PathParam<String>, depot: &mut Depot) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);

    engine.start(&project.into_inner(), None).await?;
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

    engine
        .start(&project.into_inner(), Some(&service.into_inner()))
        .await?;
    return Ok(().into());
}

/// Stops the given project and all its services.
#[endpoint]
pub async fn stop_project(project: PathParam<String>, depot: &mut Depot) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);

    engine.stop(&project.into_inner(), None).await?;
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

    engine
        .stop(&project.into_inner(), Some(&service.into_inner()))
        .await?;
    return Ok(().into());
}

/// Redeploys the given project and all its services.
#[endpoint]
pub async fn redeploy_project(project: PathParam<String>, depot: &mut Depot) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);

    engine.redeploy(&project.into_inner(), None).await?;
    return Ok(().into());
}

/// Redeploys the given service of the project.
#[endpoint]
pub async fn redeploy_service(
    project: PathParam<String>,
    service: PathParam<String>,
    depot: &mut Depot,
) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);

    engine
        .redeploy(&project.into_inner(), Some(&service.into_inner()))
        .await?;
    return Ok(().into());
}

/// Pulls the images for all the services in the given project.
#[endpoint]
pub async fn pull_project(project: PathParam<String>, depot: &mut Depot) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);

    engine.pull(&project.into_inner(), None).await?;
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

    engine
        .pull(&project.into_inner(), Some(&service.into_inner()))
        .await?;
    return Ok(().into());
}

/// Builds the images for all the services in the given project.
#[endpoint]
pub async fn build_project(project: PathParam<String>, depot: &mut Depot) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);

    engine.build(&project.into_inner(), None).await?;
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

    engine
        .build(&project.into_inner(), Some(&service.into_inner()))
        .await?;
    return Ok(().into());
}
