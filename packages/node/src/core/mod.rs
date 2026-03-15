//! The core library for Luminary, containing all logic related to managing projects and interacting with the Docker engine.

pub const COMPOSE_FILENAME: &str = "compose.yml";

mod action;
mod compose;
mod engine;
mod logs;
mod model;
mod state;

pub use engine::LuminaryEngine;
pub use model::*;
