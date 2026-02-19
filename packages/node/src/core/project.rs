//! This module implements the core logic for managing Luminary projects.

use std::{collections::HashMap, path::Path};

use bollard::{query_parameters::ListContainersOptionsBuilder, secret::ContainerSummaryStateEnum};
use color_eyre::eyre::{Ok, Result, WrapErr};
use docker_compose_types::Compose;
use luminary_macros::wrap_err;
use tokio::fs::{self, File};

use crate::core::{
    LuminaryCore,
    model::{LuminaryProject, LuminaryService, LuminaryStatus},
};

const COMPOSE_PROJECT_DIR_LABEL: &str = "com.docker.compose.project.working_dir";
const COMPOSE_PROJECT_LABEL: &str = "com.docker.compose.project";
const COMPOSE_SERVICE_LABEL: &str = "com.docker.compose.service";

const COMPOSE_FILENAME: &str = "compose.yml";

impl LuminaryCore {
    /// Lists all Luminary projects by combining data from both the filesystem and Docker engine.
    #[wrap_err("Failed to list projects")]
    pub async fn list_projects(&self) -> Result<HashMap<String, LuminaryProject>> {
        let disk_projects = self.list_from_filesystem().await?;
        let mut projects = self.list_from_docker().await?;

        for (name, disk) in disk_projects {
            let project = projects.entry(name).or_insert(disk);
            project.status = LuminaryStatus::min(project.services.values().map(|s| s.status));
        }

        return Ok(projects);
    }

    /// Lists all Luminary projects found in the configured projects directory.
    #[wrap_err("Failed to list projects from filesystem")]
    async fn list_from_filesystem(&self) -> Result<HashMap<String, LuminaryProject>> {
        let mut projects = HashMap::<String, LuminaryProject>::new();

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

    /// Parses a given compose file into a LuminaryProject struct, initializing all services with a default status of "Down".
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
                            name: name.clone(),
                            status: LuminaryStatus::Down,
                        },
                    )
                })
                .collect(),
        }
    }

    /// Lists all Luminary projects by querying the Docker engine for containers with specific labels.
    #[wrap_err("Failed to list projects from docker engine")]
    async fn list_from_docker(&self) -> Result<HashMap<String, LuminaryProject>> {
        let options = ListContainersOptionsBuilder::default().all(true).build();
        let projects = self
            .docker
            .list_containers(Some(options))
            .await
            .wrap_err("Failed to fetch containers from docker engine")?
            .iter_mut()
            .fold(HashMap::<String, LuminaryProject>::new(), |mut acc, container| {
                if let Some(mut labels) = container.labels.take()
                    && let Some(service) = labels.remove(COMPOSE_SERVICE_LABEL)
                    && let Some(project) = labels.remove(COMPOSE_PROJECT_LABEL)
                    && let Some(dir) = labels.remove(COMPOSE_PROJECT_DIR_LABEL)
                {
                    if Path::new(&dir).starts_with(&self.configuration.project_directory) {
                        let status = self.parse_state(container.state);
                        acc.entry(project.clone())
                            .or_insert_with(|| LuminaryProject {
                                services: HashMap::new(),
                                status: status,
                                name: project,
                            })
                            .services
                            .insert(
                                service.clone(),
                                LuminaryService {
                                    name: service,
                                    status,
                                },
                            );
                    }
                }

                return acc;
            });

        return Ok(projects);
    }

    /// Parses a Docker container state into a corresponding LuminaryStatus.
    fn parse_state(&self, state: Option<ContainerSummaryStateEnum>) -> LuminaryStatus {
        return match state {
            Some(ContainerSummaryStateEnum::CREATED) => LuminaryStatus::Starting,
            Some(ContainerSummaryStateEnum::RUNNING) => LuminaryStatus::Running,
            Some(ContainerSummaryStateEnum::PAUSED) => LuminaryStatus::Paused,
            Some(ContainerSummaryStateEnum::RESTARTING) => LuminaryStatus::Restarting,
            Some(ContainerSummaryStateEnum::EXITED) => LuminaryStatus::Exited,
            Some(ContainerSummaryStateEnum::REMOVING) => LuminaryStatus::Removing,
            Some(ContainerSummaryStateEnum::EMPTY) => LuminaryStatus::Down,
            Some(ContainerSummaryStateEnum::DEAD) => LuminaryStatus::Down,
            None => LuminaryStatus::Down,
        };
    }
}
