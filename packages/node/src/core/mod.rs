//! The core library for Luminary, containing all logic related to managing projects and interacting with the Docker engine.

pub const COMPOSE_FILENAME: &str = "compose.yml";
pub const COMPOSE_PROJECT_DIR_LABEL: &str = "com.docker.compose.project.working_dir";
pub const COMPOSE_PROJECT_LABEL: &str = "com.docker.compose.project";
pub const COMPOSE_SERVICE_LABEL: &str = "com.docker.compose.service";

mod action;
mod engine;
mod logs;
mod model;
mod project;
mod state;

pub use engine::LuminaryEngine;
pub use model::*;
