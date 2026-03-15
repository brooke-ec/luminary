//! Manages retrieving and updating project compose files.

use std::str::from_utf8;

use eyre::Context;
use salvo::Request;
use salvo::Writer;
use salvo::{
    Depot, Router,
    oapi::{endpoint, extract::PathParam},
};

use crate::{api::response::LuminaryResponse, core::LuminaryEngine, obtain};

/// Returns the router for compose related endpoints.
pub fn router() -> Router {
    return Router::with_path("compose").get(get_compose).put(put_compose);
}

/// Retrieves the compose file for a given project.
#[endpoint]
async fn get_compose(project: PathParam<String>, depot: &mut Depot) -> LuminaryResponse<String> {
    let engine = obtain!(depot, LuminaryEngine);
    return Ok(engine.get_compose(&project.into_inner()).await?.into());
}

/// Updates the compose file for a given project.
#[endpoint]
async fn put_compose(
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
