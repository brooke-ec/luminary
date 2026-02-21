//! This module defines the core data models used within the Luminary application.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use specta::Type;

/// A type alias for a collection of Luminary projects, keyed by project name.
pub type LuminaryProjectList = HashMap<String, LuminaryProject>;

/// Represents a Luminary project, consisting of a docker compose project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
#[serde(rename_all = "camelCase")]
pub struct LuminaryProject {
    pub name: String,
    pub status: LuminaryStatus,
    pub services: HashMap<String, LuminaryService>,
}

/// Represents a service within a Luminary project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
#[serde(rename_all = "camelCase")]
pub struct LuminaryService {
    pub id: Option<String>,
    pub name: String,
    pub status: LuminaryStatus,
}

/// Represents the various possible activity statuses of a Luminary service.
/// Variants are ordered from lowest (Exited) to highest (Healthy).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Type)]
#[serde(rename_all = "camelCase")]
pub enum LuminaryStatus {
    Exited,
    Down,
    Paused,
    Restarting,
    Removing,
    Starting,
    Running,
    Healthy,
}

impl LuminaryStatus {
    /// Returns the lowest status from an iterator, or `None` if empty.
    pub fn min(statuses: impl IntoIterator<Item = Self>) -> Self {
        statuses.into_iter().min().unwrap_or(LuminaryStatus::Down)
    }
}
