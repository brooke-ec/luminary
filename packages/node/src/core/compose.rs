use std::path::{Path, PathBuf};

use eyre::{Context, Ok, Result};
use luminary_macros::wrap_err;
use tokio::fs::read_to_string;

use crate::core::{COMPOSE_FILENAME, LuminaryEngine};

impl LuminaryEngine {
    fn compose_path(&self, project: &str) -> Result<PathBuf> {
        let path = Path::new(&self.configuration.project_directory)
            .join(project)
            .join(COMPOSE_FILENAME);

        if !path.exists() {
            eyre::bail!("Project '{}' does not exist", project);
        }

        return Ok(path);
    }

    /// Retrieves the docker compose file for a given project.
    #[wrap_err("Failed to retrieve compose file")]
    pub async fn get_compose(&self, project: &str) -> Result<String> {
        let path = self.compose_path(project)?;

        return Ok(read_to_string(path).await.wrap_err("Failed to read file")?);
    }

    /// Updates the docker compose file for a given project.
    #[wrap_err("Failed to update compose file")]
    pub async fn put_compose(&self, project: &str, compose: &str) -> Result<()> {
        let path = self.compose_path(project)?;

        tokio::fs::write(path, compose)
            .await
            .wrap_err("Failed to write file")?;

        return Ok(());
    }
}
