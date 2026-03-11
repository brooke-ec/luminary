//! This module defines the core data models used within the Luminary application.

use std::{collections::HashMap, fmt::Display};

use salvo::oapi::ToSchema;
use serde::{Serialize, ser::SerializeStruct};

/// A type alias for a collection of Luminary services, keyed by project name.
pub type LuminaryStateList = HashMap<String, LuminaryProject>;

/// Represents a Luminary project, consisting of a docker compose project.
///
/// This is derived entirely from that state of its services.
#[derive(Debug, Clone, PartialEq, ToSchema)]
pub struct LuminaryProject {
    /// The name of this project
    pub name: String,
    /// A map of the services that make up this projects
    pub services: HashMap<String, LuminaryService>,
}

impl LuminaryProject {
    /// True if any of this project's services are currently performing an action
    pub fn busy(&self) -> bool {
        self.services
            .values()
            .any(|service| service.action != LuminaryAction::Idle)
    }

    /// The minimum status of this project's services see [LuminaryStatus]
    pub fn status(&self) -> LuminaryStatus {
        LuminaryStatus::min(self.services.values().map(|service| service.status))
    }
}

impl Serialize for LuminaryProject {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("LuminaryProject", 4)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("status", &self.status())?;
        state.serialize_field("busy", &self.busy())?;
        state.serialize_field("services", &self.services)?;
        state.end()
    }
}

/// Represents a service within a Luminary project.
#[derive(Debug, Clone, Serialize, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LuminaryService {
    /// The identifier of this service
    #[serde(flatten)]
    pub identifier: LuminaryIdentifier,
    /// The current status of this service
    pub status: LuminaryStatus,
    /// The current action being performed on this service
    pub action: LuminaryAction,
    /// Wether this service is stale, meaning that it was removed in the recent refresh.
    #[serde(skip)]
    pub stale: bool,
}

/// The unique identifier for a Luminary service, consisting of its project and service names.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, ToSchema, Hash)]
#[serde(rename_all = "camelCase")]
pub struct LuminaryIdentifier {
    /// The name of the project that this service belongs to
    pub project_name: String,
    /// The name of this service
    pub service_name: String,
}

impl LuminaryIdentifier {
    /// Constructs a new [LuminaryIdentifier] from its name and its project name
    pub fn new(project_name: impl Into<String>, service_name: impl Into<String>) -> Self {
        Self {
            project_name: project_name.into(),
            service_name: service_name.into(),
        }
    }
}

impl Display for LuminaryIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.project_name, self.service_name)
    }
}

/// Represents the various possible activity statuses of a Luminary service.
/// Variants are ordered from lowest (Exited) to highest (Healthy).
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord, ToSchema)]
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
#[derive(Debug, Clone, Copy, Serialize, PartialEq, ToSchema)]
pub enum LuminaryAction {
    Idle,
    Restarting,
}
