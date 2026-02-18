use std::{collections::HashMap, path::Path};

use bollard::query_parameters::ListContainersOptionsBuilder;
use color_eyre::eyre::{ContextCompat, Ok, Result, WrapErr};
use docker_compose_types::Compose;
use luminary_macros::wrap_err;
use tokio::fs::{self, File};

use crate::core::{
    LuminaryCore,
    model::{LuminaryProject, LuminaryService},
};

const COMPOSE_PROJECT_DIR_LABEL: &str = "com.docker.compose.project.working_dir";
const COMPOSE_PROJECT_LABEL: &str = "com.docker.compose.project";
const COMPOSE_SERVICE_LABEL: &str = "com.docker.compose.service";

const COMPOSE_FILENAME: &str = "compose.yml";

impl LuminaryCore {
    #[wrap_err("Failed to list projects from filesystem")]
    async fn list_from_filesystem(&self) -> Result<HashMap<String, LuminaryProject>> {
        let mut projects = HashMap::<String, LuminaryProject>::new();

        let mut entries = fs::read_dir(&self.configuration.project_directory)
            .await
            .wrap_err("Failed to list project directory contents")?;
        while let Some(entry) = entries.next_entry().await? {
            let mut path = entry.path();
            if path.is_dir() {
                let project_name = path
                    .file_name()
                    .wrap_err("Failed to get project directory name")?
                    .to_str()
                    .wrap_err("Failed to read project directory name")?
                    .to_owned();
                path.push(COMPOSE_FILENAME);
                if path.exists() {
                    let file = File::open(path).await.wrap_err("Failed to open compose file")?;
                    let compose: Compose = serde_saphyr::from_reader(file.into_std().await)?;

                    projects.insert(
                        project_name.clone(),
                        LuminaryProject {
                            name: project_name,
                            services: compose
                                .services
                                .0
                                .into_iter()
                                .map(|(name, _)| (name.clone(), LuminaryService { name: name.clone() }))
                                .collect(),
                        },
                    );
                }
            }
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
                    if Path::new(&dir).starts_with(&self.configuration.project_directory) {
                        acc.entry(project.clone())
                            .or_insert_with(|| LuminaryProject {
                                services: HashMap::new(),
                                name: project,
                            })
                            .services
                            .insert(service.clone(), LuminaryService { name: service });
                    }
                }

                return acc;
            });

        return Ok(projects);
    }
}
