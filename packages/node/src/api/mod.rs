//! Contains all the routes for Luminary Node's API.

use eyre::Result;
use luminary_macros::wrap_err;
use salvo::{Router, affix_state, oapi::endpoint};
use sqlx::SqlitePool;

use crate::{
    api::{auth::LuminaryAuthentication, realtime::LuminaryLogsChannel},
    core::LuminaryEngine,
    util::BroadcastLayer,
};

mod actions;
mod auth;
pub mod realtime;

/// Sets up the app router and all dependencies.
#[wrap_err("Crashed while setting up")]
pub async fn setup(pool: SqlitePool, logs: BroadcastLayer) -> Result<Router> {
    let engine = LuminaryEngine::setup().await?;

    // Set up the affix state with all dependencies
    let affix = affix_state::inject(LuminaryAuthentication::new(pool.clone()))
        .inject(LuminaryLogsChannel::new(engine.clone()))
        .inject(engine)
        .inject(logs)
        .inject(pool);

    // Set up the app router
    let router = Router::new().hoop(affix).push(
        Router::with_path("/api")
            .push(auth::router())
            .push(actions::router())
            .push(realtime::router())
            .push(Router::with_path("ping").get(ping)),
    );

    // Write OpenAPI documentation to file for the panel to consume
    #[cfg(debug_assertions)]
    {
        use salvo::oapi::{
            OpenApi, SecurityScheme,
            security::{Http, HttpAuthScheme},
        };

        let openapi = OpenApi::new("Luminary Node API", env!("CARGO_PKG_VERSION"))
            .add_security_scheme("bearer", SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)))
            .merge_router(&router);

        std::fs::write(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../panel/src/lib/openapi.json"),
            openapi.to_pretty_json()?,
        )?;
    }

    return Ok(router);
}

/// A simple endpoint to test if the server is running.
#[endpoint]
async fn ping() -> &'static str {
    "pong"
}
