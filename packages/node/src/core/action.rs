use eyre::{ContextCompat, Result, bail};
use futures_util::StreamExt;
use luminary_macros::wrap_err;

use crate::core::{LuminaryAction, LuminaryEngine};

impl LuminaryEngine {
    /// Updates the currently processing action for the given project and optionally, a specific service within that project.
    #[wrap_err("Failed to set action for service")]
    pub(super) async fn set_action(
        &self,
        project: &str,
        service: Option<&str>,
        action: LuminaryAction,
    ) -> Result<()> {
        // Get list of targets to update
        let mut project_list = self.list.write().await;
        let targets = match project_list.0.get_mut(project) {
            None => bail!("Unknown project '{}'", project),
            Some(service_list) => match service {
                None => service_list.services.0.values_mut().collect(),
                Some(service) => vec![
                    service_list
                        .services
                        .0
                        .get_mut(service)
                        .wrap_err_with(|| format!("Unknown service '{}' in '{}'", service, project))?,
                ],
            },
        };

        // Check if any target is already processing an action
        if action != LuminaryAction::Idle {
            for target in &targets {
                if target.action != LuminaryAction::Idle {
                    bail!("'{}' is already processing an action", target.identifier);
                }
            }
        }

        // Set the action for all targets
        for target in targets {
            target.action = action;
        }

        self.broadcast(project_list.clone()).await;
        return Ok(());
    }

    /// A helper function to run a given command for a project and optionally, a specific service within that project.
    // TODO: Eventually we may want to stream this back to the client instead of just waiting.
    async fn run(
        &self,
        action: LuminaryAction,
        project: &str,
        service: Option<&str>,
        mut args: Vec<&str>,
    ) -> Result<()> {
        self.set_action(project, service, action).await?;

        if let Some(service) = &service {
            args.push(service);
        }

        let mut stream = self.cli(&project, args)?;
        while let Some(_) = stream.next().await {}
        self.set_action(project, service, LuminaryAction::Idle).await?;
        return Ok(());
    }

    /// Restarts the given project and optionally, a specific service within that project.
    #[wrap_err("Failed to restart project/service")]
    pub async fn restart(&self, project: &str, service: Option<&str>) -> Result<()> {
        self.run(LuminaryAction::Restarting, project, service, vec!["restart"])
            .await?;
        Ok(())
    }

    /// Starts the given project and optionally, a specific service within that project.
    #[wrap_err("Failed to start project/service")]
    pub async fn start(&self, project: &str, service: Option<&str>) -> Result<()> {
        self.run(LuminaryAction::Starting, project, service, vec!["up", "-d"])
            .await?;
        Ok(())
    }

    /// Stops the given project and optionally, a specific service within that project.
    #[wrap_err("Failed to stop project/service")]
    pub async fn stop(&self, project: &str, service: Option<&str>) -> Result<()> {
        self.run(
            LuminaryAction::Stopping,
            project,
            service,
            vec!["down", "--remove-orphans"],
        )
        .await?;
        Ok(())
    }

    /// Recreates the given project and optionally, a specific service within that project.
    #[wrap_err("Failed to recreate project/service")]
    pub async fn recreate(&self, project: &str, service: Option<&str>) -> Result<()> {
        self.stop(project, service).await?;
        self.start(project, service).await?;
        Ok(())
    }

    /// Pulls the latest images for the given project and optionally, a specific service within that project.
    #[wrap_err("Failed to pull project/service images")]
    pub async fn pull(&self, project: &str, service: Option<&str>) -> Result<()> {
        self.run(
            LuminaryAction::Pulling,
            project,
            service,
            vec!["up", "--pull", "always", "-d"],
        )
        .await?;
        Ok(())
    }

    /// Builds the images for the given project and optionally, a specific service within that project.
    #[wrap_err("Failed to build project/service images")]
    pub async fn build(&self, project: &str, service: Option<&str>) -> Result<()> {
        self.run(
            LuminaryAction::Building,
            project,
            service,
            vec!["up", "--build", "-d"],
        )
        .await?;

        Ok(())
    }
}
