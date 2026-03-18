//! Manages retrieving and updating project compose files.

use salvo::Router;
use salvo::Writer;
use salvo::oapi::ToSchema;
use salvo::oapi::extract::JsonBody;
use salvo::{
    Depot,
    oapi::{endpoint, extract::PathParam},
};
use serde::Deserialize;
use serde::Serialize;

use crate::core::LuminaryProject;
use crate::{api::response::LuminaryResponse, core::LuminaryEngine, obtain};

/// Returns the router for compose related endpoints.
pub fn router() -> Router {
    return Router::new().get(get_project_endpoint).patch(put_compose);
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
pub async fn put_compose(
    project: PathParam<String>,
    payload: JsonBody<LuminaryProjectPatch>,
    depot: &mut Depot,
) -> LuminaryResponse<LuminaryProjectWithCompose> {
    let engine = obtain!(depot, LuminaryEngine);
    let mut changed = false;

    if let Some(compose) = &payload.compose {
        engine.put_compose(&project, &compose).await?;
        changed = true;
    }

    if let Some(from) = &payload.from {
        engine.rename_project(from, &project).await?;
        changed = true;
    }

    if changed {
        engine.refresh().await?;
    }

    return get_project(engine, &project).await;
}

/// The payload for updating a project. Allows for multiple updates at once.
#[derive(Debug, Clone, Deserialize, ToSchema)]
struct LuminaryProjectPatch {
    /// If [Some], the new compose file for this project. If [None], the compose file will not be updated.
    compose: Option<String>,

    /// If [Some], renames the project with the given name. If [None], no rename will take place.
    from: Option<String>,
}
