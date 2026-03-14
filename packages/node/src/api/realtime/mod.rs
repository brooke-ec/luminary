//! Manages real-time updates

use crate::api::{
    auth::protected,
    realtime::{app::app_subscribe, logs::logs_subscribe},
};
use salvo::Router;

mod app;
mod logs;

/// Returns a router containing all realtime-related endpoints.
pub fn router() -> Router {
    return Router::with_path("/realtime")
        .hoop(protected)
        .push(Router::with_path("app").get(app_subscribe))
        .push(Router::with_path("logs").get(logs_subscribe));
}
