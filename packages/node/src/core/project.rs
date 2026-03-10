//! This module implements the core logic for managing Luminary projects.

use std::{collections::HashMap, path::Path};

use bollard::{
    query_parameters::{EventsOptionsBuilder, ListContainersOptionsBuilder},
    secret::ContainerSummaryStateEnum,
};
use bytes::Bytes;
use docker_compose_types::Compose;
use eyre::{Result, WrapErr};
use futures_util::{StreamExt, stream::BoxStream};
use log::debug;
use luminary_macros::wrap_err;
use tokio::fs::{self, File};

use crate::core::{
    LuminaryEngine, LuminaryIdentifier,
    model::{LuminaryProject, LuminaryService, LuminaryStateList, LuminaryStatus},
};

const COMPOSE_PROJECT_DIR_LABEL: &str = "com.docker.compose.project.working_dir";
const COMPOSE_PROJECT_LABEL: &str = "com.docker.compose.project";
const COMPOSE_SERVICE_LABEL: &str = "com.docker.compose.service";

const COMPOSE_FILENAME: &str = "compose.yml";

impl LuminaryEngine {
    /// Streams the application logs of the given project.
    ///
    /// This stream is infinite and will continue yield logs even if the project is restarted.
    pub fn stream_logs(&self, name: String) -> BoxStream<'_, Result<Bytes>> {
        return async_stream::stream! {
            loop {
                // Spawn docker compose process, yielding logs as they are recieved
                match self.cli(&name, ["logs", "-f"]) {
                    Err(err) => yield Err(err),
                    Ok(mut stream) => while let Some(result) = stream.next().await {
                        yield result;
                    },
                }

                // If the process exits, wait for an event from the project before triggering a retry
                debug!("Docker compose logs process exited, waiting for event to trigger retry...");

                let mut filters = HashMap::new();
                filters.insert("type", vec!["container".to_string()]);
                filters.insert("label", vec![format!("{}={}", COMPOSE_PROJECT_LABEL, &name)]);
                let options = EventsOptionsBuilder::default().filters(&filters).build();
                let mut events = self.docker.events(Some(options));
                loop {
                    let item = events.next().await.expect("Failed to receive docker event"); // Panic here as handling docker connection errors are out of scope
                    match item.wrap_err("Failed to receive Docker event") {
                        Err(err) => yield Err(err),
                        Ok(e) => {
                            if let Some(LuminaryStatus::Running) = e.action.and_then(|a| self.parse_action(a)) {
                                debug!("Received event indicating project is running, restarting logs stream...");
                                break;
                            }
                        },
                    }
                }
            }
        }
        .boxed();
    }

    /// Subscribes to Docker events and emits a partial [LuminaryProject] whenever one of its containers changes state.
    ///
    /// Partial projects can be merged into an existing [LuminaryProjectList] using [LuminaryProject::merge_into]
    ///
    /// # Examples
    /// ```
    /// use futures_util::StreamExt;
    /// use luminary_core::LuminaryProjectList;
    ///
    /// let mut stream = engine.stream();
    /// let mut state = engine.list_projects().await?;
    ///
    /// while let Some(project) = stream.next().await {
    ///     project?.merge_into(&mut state);
    ///     println!("{state:#?}");
    /// }
    /// ```
    pub fn stream_updates(&self) -> BoxStream<'_, Result<LuminaryProject>> {
        let mut filters = HashMap::new();
        filters.insert("type", vec!["container"]);
        let options = EventsOptionsBuilder::default().filters(&filters).build();

        debug!("Subscribing to Docker events for project updates");

        let docker_stream = self.docker.events(Some(options)).filter_map(move |event| async {
            match event.wrap_err("Failed to receive Docker event") {
                Err(err) => Some(Err(err)),
                Ok(event) => {
                    if let Some(actor) = event.actor
                        && let Some(action) = event.action
                        && let Some(labels) = actor.attributes
                        && let Some(status) = self.parse_action(action.clone())
                        && let Some(project) = self.parse_labels(status, labels).await
                    {
                        return Some(Ok(project));
                    } else {
                        return None;
                    }
                }
            }
        });

        let mut action_channel = self.actions_channel.subscribe();
        let action_stream = async_stream::stream! {
            loop {
                match action_channel.recv().await {
                    Ok(project) => yield Ok(project),
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        };

        return futures_util::stream::select(docker_stream, action_stream).boxed();
    }

    /// Lists all Luminary projects by combining data from both the filesystem and Docker engine.
    #[wrap_err("Failed to list projects")]
    pub async fn list_projects(&self) -> Result<LuminaryStateList> {
        let disk_projects = self.list_from_filesystem().await?;
        let mut projects = self.list_from_docker().await?;

        for (name, disk) in disk_projects {
            projects.entry(name).or_insert(disk);
        }

        return Ok(projects);
    }

    /// Lists all Luminary projects found in the configured projects directory.
    #[wrap_err("Failed to list projects from filesystem")]
    async fn list_from_filesystem(&self) -> Result<LuminaryStateList> {
        let mut services = LuminaryStateList::new();

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
                    services.extend(self.parse_compose(project_name, compose).await);
                }
            }
        }

        return Ok(services);
    }

    /// Lists all Luminary projects by querying the Docker engine for containers with specific labels.
    #[wrap_err("Failed to list projects from docker engine")]
    async fn list_from_docker(&self) -> Result<LuminaryStateList> {
        let options = ListContainersOptionsBuilder::default().all(true).build();
        let containers = self
            .docker
            .list_containers(Some(options))
            .await
            .wrap_err("Failed to fetch containers from docker engine")?;

        let mut projects = LuminaryStateList::new();
        for mut container in containers {
            if let Some(labels) = container.labels.take() {
                if let Some(project) = self.parse_labels(self.parse_state(container.state), labels).await {
                    project.merge_into(&mut projects);
                }
            }
        }

        return Ok(projects);
    }

    /// Parses a given compose file into a LuminaryProject struct, initializing all services with a default status of "Down".
    async fn parse_compose(&self, name: String, compose: Compose) -> LuminaryStateList {
        let mut services = LuminaryStateList::new();

        for (service_name, _) in compose.services.0 {
            let identifier = LuminaryIdentifier::new(name, service_name);
            services.insert(
                identifier.clone(),
                LuminaryService {
                    action: self.get_action(identifier).await,
                    status: LuminaryStatus::Down,
                    identifier,
                },
            );
        }

        return services;
    }

    /// Parses Docker container labels into a LuminaryProject, if the required labels are present.
    async fn parse_labels(
        &self,
        status: LuminaryStatus,
        mut labels: HashMap<String, String>,
    ) -> Option<LuminaryService> {
        if let Some(service) = labels.remove(COMPOSE_SERVICE_LABEL)
            && let Some(project) = labels.remove(COMPOSE_PROJECT_LABEL)
            && let Some(dir) = labels.remove(COMPOSE_PROJECT_DIR_LABEL)
            && Path::new(&dir).starts_with(&self.configuration.project_directory)
        {
            let identifier = LuminaryIdentifier::new(project, service);
            return Some(LuminaryService {
                action: self.get_action(&identifier).await,
                identifier,
                status,
            });
        }

        return None;
    }

    /// Parses a Docker container state into a corresponding LuminaryStatus.
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

    /// Translate a Docker event action into a LuminaryStatus, if possible.
    // Event list taken from https://docs.docker.com/engine/reference/commandline/events/#events-list
    fn parse_action(&self, action: String) -> Option<LuminaryStatus> {
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
}

impl LuminaryProject {
    /// Merges this project into the provided list, combining with existing services.
    pub fn merge_into(self, list: &mut LuminaryStateList) {
        if let Some(existing) = list.get_mut(&self.name) {
            for (service_name, service) in self.services {
                existing.services.insert(service_name, service);
            }

            existing.action = self.action;
            existing.status = LuminaryStatus::min(existing.services.values().map(|s| s.status));
        } else {
            list.insert(self.name.clone(), self);
        }
    }
}
