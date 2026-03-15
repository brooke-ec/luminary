//! Contains all the routes for Luminary Node's API.

use eyre::Result;
use luminary_macros::wrap_err;
use salvo::{Router, affix_state, oapi::endpoint};
use sqlx::SqlitePool;

use crate::{
    api::{
        auth::{LuminaryAuthentication, protected},
        realtime::{app_subscribe, logs_subscribe},
    },
    core::LuminaryEngine,
    logging::BroadcastLayer,
};

mod actions;
mod auth;
pub mod realtime;
mod response;

/// Sets up the app router and all dependencies.
#[wrap_err("Crashed while setting up")]
pub async fn setup(pool: SqlitePool, logs: BroadcastLayer) -> Result<Router> {
    let engine = LuminaryEngine::setup().await?;

    // Set up the affix state with all dependencies
    let affix = affix_state::inject(LuminaryAuthentication::new(pool.clone()))
        .inject(engine)
        .inject(logs)
        .inject(pool);

    // Set up the app router
    let router = Router::new().hoop(affix).push(
        Router::with_path("/api")
            .push(Router::with_path("ping").get(ping))
            .push(auth::router())
            .push(
                Router::new()
                    .hoop(protected)
                    .push(Router::with_path("realtime").get(app_subscribe))
                    .push(
                        Router::with_path("/project/{project}")
                            .push(Router::with_path("logs").get(logs_subscribe))
                            .push(actions::router())
                            .push(Router::with_path("service/{service}").push(actions::router())),
                    ),
            ),
    );

    // Write OpenAPI documentation to file for the panel to consume
    #[cfg(debug_assertions)]
    {
        use log::info;
        use salvo::oapi::{
            OpenApi, SecurityScheme, ToSchema,
            security::{Http, HttpAuthScheme},
        };

        info!("Generating OpenAPI documentation...");

        let mut openapi = OpenApi::new("Luminary Node API", env!("CARGO_PKG_VERSION"))
            .add_security_scheme("bearer", SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)))
            .merge_router(&router);

        // Ensure custom core schemas are registered for SSE documentation.
        crate::core::LuminaryStateList::to_schema(&mut openapi.components);

        let location = concat!(env!("CARGO_MANIFEST_DIR"), "/../panel/static/openapi.json");
        std::fs::write(location, openapi.to_pretty_json()?)?;
        info!("OpenAPI documentation written to {}", location);
    }

    return Ok(router);
}

/// A simple endpoint to test if the server is running.
#[endpoint]
async fn ping() -> &'static str {
    "pong"
}
