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
    return Router::new().get(get_project).put(put_compose);
}

/// Retrieves the compose file for a given project.
#[endpoint]
pub async fn get_project(
    project: PathParam<String>,
    depot: &mut Depot,
) -> LuminaryResponse<LuminaryProjectWithCompose> {
    let engine = obtain!(depot, LuminaryEngine);

    let name = project.into_inner();
    let compose = engine.get_compose(&name).await?;
    let project = engine.get_project(&name).await?;

    return Ok(LuminaryProjectWithCompose { project, compose }.into());
}

#[derive(Debug, Clone, Serialize, ToSchema)]
struct LuminaryProjectWithCompose {
    #[serde(flatten)]
    project: LuminaryProject,

    /// The compose file for this project.
    compose: String,
}

/// Updates the compose file for a given project.
#[endpoint]
pub async fn put_compose(
    project: PathParam<String>,
    payload: JsonBody<ComposeWithName>,
    depot: &mut Depot,
) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);

    engine
        .put_compose(&project.into_inner(), &payload.compose)
        .await?;

    return Ok(().into());
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
struct ComposeWithName {
    /// The new name for the project.
    name: String,

    /// The compose file for this project.
    compose: String,
}
