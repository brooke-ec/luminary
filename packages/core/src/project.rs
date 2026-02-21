//! This module implements the core logic for managing Luminary projects.

use std::{collections::HashMap, path::Path};

use bollard::{query_parameters::ListContainersOptionsBuilder, secret::ContainerSummaryStateEnum};
use docker_compose_types::Compose;
use eyre::{Ok, Result, WrapErr};
use luminary_macros::wrap_err;
use tokio::fs::{self, File};

use crate::{
    LuminaryEngine,
    model::{LuminaryProject, LuminaryProjectList, LuminaryService, LuminaryStatus},
};

const COMPOSE_PROJECT_DIR_LABEL: &str = "com.docker.compose.project.working_dir";
const COMPOSE_PROJECT_LABEL: &str = "com.docker.compose.project";
const COMPOSE_SERVICE_LABEL: &str = "com.docker.compose.service";

const COMPOSE_FILENAME: &str = "compose.yml";

impl LuminaryEngine {
    /// Lists all Luminary projects by combining data from both the filesystem and Docker engine.
    #[wrap_err("Failed to list projects")]
    pub async fn list_projects(&self) -> Result<LuminaryProjectList> {
        let disk_projects = self.list_from_filesystem().await?;
        let mut projects = self.list_from_docker().await?;

        for (name, disk) in disk_projects {
            projects.entry(name).or_insert(disk);
        }

        return Ok(projects);
    }

    /// Lists all Luminary projects found in the configured projects directory.
    #[wrap_err("Failed to list projects from filesystem")]
    async fn list_from_filesystem(&self) -> Result<LuminaryProjectList> {
        let mut projects = LuminaryProjectList::new();

        let mut entries = fs::read_dir(&self.configuration.project_directory)
            .await
            .wrap_err("Failed to list project directory contents")?;
        while let Some(entry) = entries.next_entry().await? {
            let mut path = entry.path();
            if path.is_dir()
                && let Some(project_name) = path.file_name().and_then(|n| n.to_str()).map(|s| s.to_string())
            {
                path.push(COMPOSE_FILENAME);
                if path.exists() {
                    let file = File::open(path).await.wrap_err("Failed to open compose file")?;
                    let compose: Compose = serde_saphyr::from_reader(file.into_std().await)?;
                    projects.insert(project_name.clone(), self.parse_compose(project_name, compose));
                }
            }
        }

        return Ok(projects);
    }

    /// Lists all Luminary projects by querying the Docker engine for containers with specific labels.
    #[wrap_err("Failed to list projects from docker engine")]
    async fn list_from_docker(&self) -> Result<LuminaryProjectList> {
        let options = ListContainersOptionsBuilder::default().all(true).build();
        let projects = self
            .docker
            .list_containers(Some(options))
            .await
            .wrap_err("Failed to fetch containers from docker engine")?
            .iter_mut()
            .fold(LuminaryProjectList::new(), |mut acc, container| {
                if let Some(labels) = container.labels.take() {
                    if let Some(project) = self.parse_labels(self.parse_state(container.state), labels) {
                        acc.merge_project(project);
                    }
                }

                return acc;
            });

        return Ok(projects);
    }
}

pub trait LuminaryProjectParser {
    /// Parses a given compose file into a LuminaryProject struct, initializing all services with a default status of "Down".
    fn parse_compose(&self, name: String, compose: Compose) -> LuminaryProject;

    /// Parses container labels to determine if they correspond to a Luminary project, if so, constructs a LuminaryProject struct from them.
    fn parse_labels(
        &self,
        status: LuminaryStatus,
        labels: HashMap<String, String>,
    ) -> Option<LuminaryProject>;

    /// Parses a Docker container state into a corresponding LuminaryStatus.
    fn parse_state(&self, state: Option<ContainerSummaryStateEnum>) -> LuminaryStatus;
}

impl LuminaryProjectParser for LuminaryEngine {
    fn parse_compose(&self, name: String, compose: Compose) -> LuminaryProject {
        LuminaryProject {
            name,
            status: LuminaryStatus::Down,
            services: compose
                .services
                .0
                .into_iter()
                .map(|(name, _)| {
                    (
                        name.clone(),
                        LuminaryService {
                            status: LuminaryStatus::Down,
                            name,
                        },
                    )
                })
                .collect(),
        }
    }

    fn parse_labels(
        &self,
        status: LuminaryStatus,
        mut labels: HashMap<String, String>,
    ) -> Option<LuminaryProject> {
        if let Some(service) = labels.remove(COMPOSE_SERVICE_LABEL)
            && let Some(project) = labels.remove(COMPOSE_PROJECT_LABEL)
            && let Some(dir) = labels.remove(COMPOSE_PROJECT_DIR_LABEL)
            && Path::new(&dir).starts_with(&self.configuration.project_directory)
        {
            return Some(LuminaryProject {
                name: project,
                status,
                services: HashMap::from([(
                    service.clone(),
                    LuminaryService {
                        name: service,
                        status,
                    },
                )]),
            });
        }

        return None;
    }

    fn parse_state(&self, state: Option<ContainerSummaryStateEnum>) -> LuminaryStatus {
        return match state {
            Some(ContainerSummaryStateEnum::CREATED) => LuminaryStatus::Loading,
            Some(ContainerSummaryStateEnum::RUNNING) => LuminaryStatus::Running,
            Some(ContainerSummaryStateEnum::PAUSED) => LuminaryStatus::Paused,
            Some(ContainerSummaryStateEnum::RESTARTING) => LuminaryStatus::Loading,
            Some(ContainerSummaryStateEnum::EXITED) => LuminaryStatus::Exited,
            Some(ContainerSummaryStateEnum::REMOVING) => LuminaryStatus::Loading,
            Some(ContainerSummaryStateEnum::EMPTY) => LuminaryStatus::Down,
            Some(ContainerSummaryStateEnum::DEAD) => LuminaryStatus::Down,
            None => LuminaryStatus::Down,
        };
    }
}

pub trait LuminaryProjectListExt {
    /// Adds a project to the list, merging with any existing project of the same name.
    fn merge_project(&mut self, b: LuminaryProject);
}

impl LuminaryProjectListExt for LuminaryProjectList {
    fn merge_project(&mut self, project: LuminaryProject) {
        if let Some(existing) = self.get_mut(&project.name) {
            for (service_name, service) in project.services {
                existing.services.insert(service_name, service);
            }

            existing.status = LuminaryStatus::min(existing.services.values().map(|s| s.status));
        } else {
            self.insert(project.name.clone(), project);
        }
    }
}
