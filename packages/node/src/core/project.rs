use std::collections::HashMap;

use bollard::query_parameters::ListContainersOptionsBuilder;
use color_eyre::eyre::{Ok, Result, WrapErr};
use luminary_macros::wrap_err;
use tokio::fs;

use crate::core::{
    LuminaryCore,
    model::{LuminaryProject, LuminaryService},
    project,
};

const COMPOSE_PROJECT_DIR_LABEL: &str = "com.docker.compose.project.working_dir";
const COMPOSE_PROJECT_LABEL: &str = "com.docker.compose.project";
const COMPOSE_SERVICE_LABEL: &str = "com.docker.compose.service";

const ENV_PROJECT_DIRECTORY: &str = "LUMINARY_PROJECT_DIRECTORY";

impl LuminaryCore {
    #[wrap_err("Failed to list projects from filesystem")]
    async fn list_from_filesystem(&self) -> Result<HashMap<String, LuminaryProject>> {
        let dir = std::env::var(ENV_PROJECT_DIRECTORY)
            .wrap_err("Failed to read project directory from environment variable")?;

        let mut projects = HashMap::<String, LuminaryProject>::new();

        let mut entries = fs::read_dir(dir)
            .await
            .wrap_err("Failed to list project directory contents")?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            // do something with path
        }

        return Ok(projects);
    }

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
                    acc.entry(project.clone())
                        .or_insert_with(|| LuminaryProject {
                            services: HashMap::new(),
                            name: project,
                            dir,
                        })
                        .services
                        .insert(service.clone(), LuminaryService { name: service });
                }

                return acc;
            });

        return Ok(projects);
    }
}
