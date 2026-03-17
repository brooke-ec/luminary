//! Manages retrieving and updating project compose files.

use std::str::from_utf8;

use eyre::Context;
use salvo::Request;
use salvo::Router;
use salvo::Writer;
use salvo::oapi::ToSchema;
use salvo::{
    Depot,
    oapi::{endpoint, extract::PathParam},
};
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
    req: &mut Request,
    depot: &mut Depot,
) -> LuminaryResponse<()> {
    let engine = obtain!(depot, LuminaryEngine);
    let bytes = req.payload().await.wrap_err("Failed to read request body")?;
    let compose = from_utf8(bytes).wrap_err("Failed to decode error")?;
    engine.put_compose(&project.into_inner(), compose).await?;
    return Ok(().into());
}
