//! This module implements the core logic for managing Luminary projects.

use std::{collections::HashMap, path::Path};

use bollard::{
    query_parameters::{EventsOptionsBuilder, ListContainersOptionsBuilder},
    secret::ContainerSummaryStateEnum,
};
use docker_compose_types::Compose;
use eyre::{Result, WrapErr};
use futures_util::StreamExt;
use log::{debug, error, warn};
use luminary_macros::wrap_err;
use tokio::fs::{self, File};

use crate::core::{
    LuminaryAction, LuminaryEngine, LuminaryIdentifier,
    model::{LuminaryProject, LuminaryService, LuminaryStateList, LuminaryStatus},
};

const COMPOSE_PROJECT_DIR_LABEL: &str = "com.docker.compose.project.working_dir";
const COMPOSE_PROJECT_LABEL: &str = "com.docker.compose.project";
const COMPOSE_SERVICE_LABEL: &str = "com.docker.compose.service";

const COMPOSE_FILENAME: &str = "compose.yml";

impl LuminaryEngine {
    pub(super) async fn spawn_state_worker(&self) {
        let this = self.clone();
        let mut filters = HashMap::new();
        filters.insert("type", vec!["container"]);

        tokio::spawn(async move {
            loop {
                debug!("Starting Docker event listener");
                let options = EventsOptionsBuilder::default().filters(&filters).build();
                let mut stream = this.docker.events(Some(options));

                while let Some(event) = stream.next().await {
                    match event {
                        Err(err) => error!("Error receiving Docker event: {:?}", err),
                        Ok(event) => {
                            if let Some(actor) = event.actor
                                && let Some(action) = event.action
                                && let Some(labels) = actor.attributes
                                && let Some(status) = Self::parse_action(action.clone())
                            {
                                let mut list = this.state.write().await;
                                this.merge_service(status, labels, &mut list);
                                this.broadcast(list.clone()).await;
                            }
                        }
                    }
                }

                warn!("Docker event stream ended, restarting...");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });
    }

    /// Lists all Luminary projects by combining data from both the filesystem and Docker engine.
    #[wrap_err("Failed to list projects")]
    pub async fn refresh(&self) -> Result<()> {
        let mut list = self.state.write().await;

        for project in list.values_mut() {
            for service in project.services.values_mut() {
                service.stale = true;
            }
        }

        self.load_from_docker(&mut list).await?;
        self.load_from_filesystem(&mut list).await?;

        list.retain(|_, project| {
            project.services.retain(|_, service| !service.stale);
            return !project.services.is_empty();
        });

        self.broadcast(list.clone()).await;
        return Ok(());
    }

    /// Loads all Luminary projects found in the configured projects directory into the given state list.
    #[wrap_err("Failed to load projects from filesystem")]
    async fn load_from_filesystem(&self, list: &mut LuminaryStateList) -> Result<()> {
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
                    let file = File::open(path)
                        .await
                        .wrap_err("Failed to open compose file")?
                        .into_std()
                        .await;

                    // Run this in a thread as it uses a blocking file reader instead of an async one
                    let compose: Compose = tokio::task::spawn_blocking(move || {
                        return serde_saphyr::from_reader(file);
                    })
                    .await
                    .wrap_err("Compose deserialization failed.")?
                    .wrap_err_with(|| format!("Failed to deserialize compose file for {}", &project_name))?;

                    let project = list
                        .entry(project_name.clone())
                        .or_insert_with(|| LuminaryProject {
                            name: project_name.clone(),
                            services: HashMap::new(),
                        });

                    for (service_name, _) in compose.services.0 {
                        let existing = project.services.get(&service_name);
                        project.services.insert(
                            service_name.clone(),
                            LuminaryService {
                                stale: false,
                                action: existing.map(|s| s.action).unwrap_or(LuminaryAction::Idle),
                                status: existing.map(|s| s.status).unwrap_or(LuminaryStatus::Down),
                                identifier: LuminaryIdentifier::new(project_name.clone(), service_name),
                            },
                        );
                    }
                }
            }
        }

        return Ok(());
    }

    /// Lists all Luminary projects by querying the Docker engine for containers with specific labels.
    #[wrap_err("Failed to list projects from docker engine")]
    async fn load_from_docker(&self, list: &mut LuminaryStateList) -> Result<()> {
        let options = ListContainersOptionsBuilder::default().all(true).build();
        let containers = self
            .docker
            .list_containers(Some(options))
            .await
            .wrap_err("Failed to fetch containers from docker engine")?;

        for mut container in containers {
            if let Some(labels) = container.labels.take() {
                self.merge_service(Self::parse_state(container.state), labels, list);
            }
        }

        return Ok(());
    }

    fn merge_service(
        &self,
        status: LuminaryStatus,
        mut labels: HashMap<String, String>,
        list: &mut LuminaryStateList,
    ) {
        if let Some(service_name) = labels.remove(COMPOSE_SERVICE_LABEL)
            && let Some(project_name) = labels.remove(COMPOSE_PROJECT_LABEL)
            && let Some(dir) = labels.remove(COMPOSE_PROJECT_DIR_LABEL)
            && Path::new(&dir).starts_with(&self.configuration.project_directory)
        {
            let project = list
                .entry(project_name.clone())
                .or_insert_with(|| LuminaryProject {
                    name: project_name.clone(),
                    services: HashMap::new(),
                });

            let existing = project.services.get(&service_name);
            project.services.insert(
                service_name.clone(),
                LuminaryService {
                    action: existing.map(|s| s.action).unwrap_or(LuminaryAction::Idle),
                    identifier: LuminaryIdentifier::new(project_name, service_name),
                    stale: false,
                    status,
                },
            );
        }
    }

    /// Parses a Docker container state into a corresponding LuminaryStatus.
    fn parse_state(state: Option<ContainerSummaryStateEnum>) -> LuminaryStatus {
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

    /// Translate a Docker event action into a LuminaryStatus, if possible.
    // Event list taken from https://docs.docker.com/engine/reference/commandline/events/#events-list
    fn parse_action(action: String) -> Option<LuminaryStatus> {
        return match action.as_str() {
            "create" => Some(LuminaryStatus::Loading),
            "destroy" => Some(LuminaryStatus::Down),
            "start" => Some(LuminaryStatus::Running),
            "stop" => Some(LuminaryStatus::Exited),
            "restart" => Some(LuminaryStatus::Running),
            "kill" => Some(LuminaryStatus::Loading),
            "pause" => Some(LuminaryStatus::Paused),
            "unpause" => Some(LuminaryStatus::Running),
            "die" => Some(LuminaryStatus::Exited),
            "oom" => Some(LuminaryStatus::Exited),
            _ => None,
        };
    }

    /// Waits until a given project or service reaches the desired status by listening to Docker events.
    pub(super) async fn wait_until(
        &self,
        project: &String,
        service: Option<&String>,
        desired: LuminaryStatus,
    ) -> Result<()> {
        // Set up filters based on arguments
        let mut filters = HashMap::new();
        filters.insert("type", vec!["container".to_string()]);
        filters.insert("label", vec![format!("{}={}", COMPOSE_PROJECT_LABEL, &project)]);
        if let Some(service) = service {
            filters.insert("label", vec![format!("{}={}", COMPOSE_SERVICE_LABEL, &service)]);
        }

        let options = EventsOptionsBuilder::default().filters(&filters).build();
        let mut events = self.docker.events(Some(options));

        // Outer loop to restart event stream if it ends unexpectedly
        loop {
            // Inner loop to wait for desired event
            loop {
                if let Some(true) = match events.next().await {
                    Some(Err(err)) => return Err(eyre::eyre!("Failed to receive Docker event: {:?}", err)),
                    Some(Ok(e)) => e.action.and_then(|a| Self::parse_action(a)).map(|a| a == desired),
                    None => break, // Break inner loop to restart event stream if it ends unexpectedly
                } {
                    return Ok(());
                }
            }

            warn!("Docker event stream ended, restarting...");
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
}
