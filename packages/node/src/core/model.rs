//! This module defines the core data models used within the Luminary application.

use std::collections::HashMap;

use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};

/// A type alias for a collection of Luminary projects, keyed by project name.
pub type LuminaryProjectList = HashMap<String, LuminaryProject>;

/// Represents a Luminary project, consisting of a docker compose project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LuminaryProject {
    pub name: String,
    pub status: LuminaryStatus,
    pub action: LuminaryAction,
    pub services: HashMap<String, LuminaryService>,
}

/// Represents a service within a Luminary project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LuminaryService {
    pub name: String,
    pub status: LuminaryStatus,
}

/// Represents the various possible activity statuses of a Luminary service.
/// Variants are ordered from lowest (Exited) to highest (Healthy).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum LuminaryStatus {
    /// Represents a service that has exited (usually due to an error).
    Exited,

    /// Represents a service that is offline.
    Down,

    /// Represents a service that has been paused.
    Paused,

    /// Represents a service that is in the process of changing state.
    Loading,

    /// Represents a service that is running and online.
    Running,

    /// Represents a service that is actively passing health checks.
    Healthy,
}

impl LuminaryStatus {
    /// Returns the lowest status from an iterator, or `None` if empty.
    pub fn min(statuses: impl IntoIterator<Item = Self>) -> Self {
        statuses.into_iter().min().unwrap_or(LuminaryStatus::Down)
    }
}

/// Represents the current action running on a Luminary project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum LuminaryAction {
    Idle,
    Restarting,
}
