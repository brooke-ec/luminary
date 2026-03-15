use eyre::{ContextCompat, Result, bail};
use futures_util::StreamExt;
use luminary_macros::wrap_err;

use crate::core::{LuminaryAction, LuminaryEngine};

impl LuminaryEngine {
    /// Updates the currently processing action for the given project and optionally, a specific service within that project.
    #[wrap_err("Failed to set action for service")]
    async fn set_action(
        &self,
        project: String,
        service: Option<String>,
        action: LuminaryAction,
    ) -> Result<()> {
        // Get list of targets to update
        let mut project_list = self.state.write().await;
        let targets = match project_list.0.get_mut(&project) {
            None => bail!("Unknown project '{}'", project),
            Some(service_list) => match service {
                None => service_list.services.0.values_mut().collect(),
                Some(service) => vec![
                    service_list
                        .services
                        .0
                        .get_mut(&service)
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
        project: String,
        service: Option<String>,
        mut args: Vec<&str>,
    ) -> Result<()> {
        self.set_action(project.clone(), service.clone(), action).await?;

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
    pub async fn restart(&self, project: String, service: Option<String>) -> Result<()> {
        self.run(LuminaryAction::Restarting, project, service, vec!["restart"])
            .await?;
        Ok(())
    }

    /// Starts the given project and optionally, a specific service within that project.
    #[wrap_err("Failed to start project/service")]
    pub async fn start(&self, project: String, service: Option<String>) -> Result<()> {
        self.run(LuminaryAction::Starting, project, service, vec!["up", "-d"])
            .await?;
        Ok(())
    }

    /// Stops the given project and optionally, a specific service within that project.
    #[wrap_err("Failed to stop project/service")]
    pub async fn stop(&self, project: String, service: Option<String>) -> Result<()> {
        self.run(LuminaryAction::Stopping, project, service, vec!["down"])
            .await?;
        Ok(())
    }
}
