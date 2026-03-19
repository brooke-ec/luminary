//! Manages retrieving and updating project compose files.

use salvo::Router;
use salvo::Writer;
use salvo::oapi::ToSchema;
use salvo::oapi::extract::JsonBody;
use salvo::{
    Depot,
    oapi::{endpoint, extract::PathParam},
};
use serde::Serialize;

use crate::core::LuminaryProject;
use crate::core::LuminaryProjectPatch;
use crate::{api::response::LuminaryResponse, core::LuminaryEngine, obtain};

/// Returns the router for compose related endpoints.
pub fn router() -> Router {
    return Router::new()
        .get(get_project_endpoint)
        .patch(patch_compose)
        .delete(delete_project);
}

/// Retrieves the compose file for a given project.
#[endpoint]
pub async fn get_project_endpoint(
    project: PathParam<String>,
    depot: &mut Depot,
) -> LuminaryResponse<LuminaryProjectWithCompose> {
    let engine = obtain!(depot, LuminaryEngine);

    return get_project(engine, &project).await;
}

/// Retrieves the compose file and information for a given project.
async fn get_project(engine: &LuminaryEngine, name: &String) -> LuminaryResponse<LuminaryProjectWithCompose> {
    let compose = engine.get_compose(name).await?;
    let project = engine.get_project(name).await?;

    return Ok(LuminaryProjectWithCompose { project, compose }.into());
}

/// A project with its compose file. Used for returning project information in a single request.
#[derive(Debug, Clone, Serialize, ToSchema)]
struct LuminaryProjectWithCompose {
    #[serde(flatten)]
    project: LuminaryProject,

    /// The compose file for this project.
    compose: String,
}

/// Perform a configurable change to a project.
#[endpoint]
pub async fn patch_compose(
    project: PathParam<String>,
    mut payload: JsonBody<LuminaryProjectPatch>,
    depot: &mut Depot,
) -> LuminaryResponse<LuminaryProjectWithCompose> {
    let engine = obtain!(depot, LuminaryEngine);

    if !payload.creating {
        engine.wait_until_idle(&project, None).await?;
    }

    engine.patch_project(&project, &payload.0).await?;
    return get_project(engine, &payload.to.take().unwrap_or(project.0)).await;
}

/// Deletes a project and all of its resources.
#[endpoint]
pub async fn delete_project(project: PathParam<String>, depot: &mut Depot) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);

    engine.wait_until_idle(&project, None).await?;
    engine.delete_project(&project).await?;

    return Ok(().into());
}
