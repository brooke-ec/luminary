use std::path::{Path, PathBuf};

use eyre::{Context, Ok, Result};
use futures_util::StreamExt;
use luminary_macros::wrap_err;
use tokio::fs::{self, read_to_string};

use crate::core::{COMPOSE_FILENAME, LuminaryAction, LuminaryEngine, LuminaryProjectPatch, LuminaryStatus};

impl LuminaryEngine {
    /// Retrieves the paths for the given project directory and its compose file.
    // TODO: In future this should look up the project directory from the program state
    fn get_path(&self, project: &str) -> (PathBuf, PathBuf) {
        let project_path = Path::new(&self.configuration.project_directory).join(project);

        let compose_path = project_path.join(COMPOSE_FILENAME);

        return (project_path, compose_path);
    }

    /// Retrieves the docker compose file for a given project.
    #[wrap_err("Failed to retrieve compose file")]
    pub async fn get_compose(&self, project: &str) -> Result<String> {
        let (_, path) = self.get_path(project);

        if !path.exists() {
            eyre::bail!("Project '{}' does not exist", project);
        }

        return Ok(read_to_string(path).await.wrap_err("Failed to read file")?);
    }

    /// Updates the given project by applying the provided patch
    pub async fn patch_project(&self, project: &str, patch: &LuminaryProjectPatch) -> Result<()> {
        self.set_action(project, None, LuminaryAction::Patching).await?;

        let (project_path, compose_path) = self.get_path(project);
        let mut changed = false;

        if let Some(compose) = &patch.compose {
            self.put_compose(&project_path, &compose_path, &compose).await?;
            changed = true;
        }

        if let Some(to) = &patch.to {
            self.rename_project(&project, &project_path, to).await?;
            changed = true;
        }

        self.set_action(project, None, LuminaryAction::Idle).await?;

        if changed {
            self.refresh().await?;
        }

        return Ok(());
    }

    /// Updates the docker compose file for a given project.
    /// WARNING: Does not automatically call `refresh`, make sure to do this after calling this function.
    #[wrap_err("Failed to update compose file")]
    async fn put_compose(&self, project_path: &PathBuf, compose_path: &PathBuf, compose: &str) -> Result<()> {
        // Create project directory if it doesn't exist
        fs::create_dir_all(&project_path)
            .await
            .wrap_err("Failed to create project directory")?;

        // Write compose file
        fs::write(compose_path, compose)
            .await
            .wrap_err("Failed to write file")?;

        return Ok(());
    }

    /// Renames the project dirctory for a given project. Recreating the project if it was previously running.
    /// WARNING: Does not automatically call `refresh`, make sure to do this after calling this function.
    #[wrap_err("Failed to rename project")]
    async fn rename_project(&self, from: &str, from_path: &PathBuf, to: &str) -> Result<()> {
        let to_path = from_path.parent().unwrap().join(to);

        if !from_path.exists() {
            eyre::bail!("Project '{}' does not exist", from);
        }

        if to_path.exists() {
            eyre::bail!("There is already a project with the name '{}'", to);
        }

        // Labels are immutable so recreate the project to update Docker Compose labels (if it was not down).
        let recreate = self
            .get_project(from)
            .await?
            .services
            .0
            .iter()
            .any(|(_, s)| s.status != LuminaryStatus::Down);

        if recreate {
            // Use manual command as action is currently "patching"
            let mut stream = self.cli(&from, vec!["down", "--remove-orphans"])?;
            while let Some(_) = stream.next().await {}
        };

        fs::rename(&from_path, &to_path)
            .await
            .wrap_err("Failed to rename project directory")?;

        if recreate {
            let mut stream = self.cli(&to, vec!["up", "-d"])?;
            while let Some(_) = stream.next().await {}
        }

        return Ok(());
    }
}
