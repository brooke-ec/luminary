use std::collections::HashMap;

use bollard::query_parameters::ListContainersOptionsBuilder;
use color_eyre::eyre::{Ok, Result};

use crate::core::{
    LuminaryCore,
    model::{LuminaryProject, LuminaryService},
};

const COMPOSE_PROJECT_DIR_LABEL: &str = "com.docker.compose.project.working_dir";
const COMPOSE_PROJECT_LABEL: &str = "com.docker.compose.project";
const COMPOSE_SERVICE_LABEL: &str = "com.docker.compose.service";

impl LuminaryCore {
    pub async fn get_projects(&self) -> Result<HashMap<String, LuminaryProject>> {
        let options = ListContainersOptionsBuilder::default().all(true).build();
        let projects = self.docker.list_containers(Some(options)).await?.iter_mut().fold(
            HashMap::<String, LuminaryProject>::new(),
            |mut acc, container| {
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
            },
        );

        return Ok(projects);
    }
}
