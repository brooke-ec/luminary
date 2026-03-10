use eyre::{ContextCompat, Result, bail};
use futures_util::{Stream, StreamExt};
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
        let targets = match project_list.get_mut(&project) {
            None => bail!("Unknown project '{}'", project),
            Some(service_list) => match service {
                None => service_list.services.values_mut().collect(),
                Some(service) => vec![
                    service_list
                        .services
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

            // Update the state for any listeners
            if self.channel.receiver_count() > 0 {
                let _ = self.channel.send(target.clone()); // This will only error if there are no receivers, so we can safely ignore it.
            }
        }

        return Ok(());
    }

    /// Waits for the given stream to complete and sets the project's action to Idle once done.
    /// TODO: Eventually we may want to stream this back to the client instead of just waiting.
    fn wait<T>(
        &self,
        project: String,
        service: Option<String>,
        mut stream: impl Stream<Item = T> + Unpin + Send + 'static,
    ) -> tokio::task::JoinHandle<()> {
        let this = self.clone();
        return tokio::spawn(async move {
            while let Some(_) = stream.next().await {}
            this.set_action(project, service, LuminaryAction::Idle).await;
        });
    }

    /// Restarts the given project and optionally, a specific service within that project.
    pub async fn restart(
        &self,
        project: String,
        service: Option<String>,
    ) -> Result<tokio::task::JoinHandle<()>> {
        self.set_action(project.clone(), service.clone(), LuminaryAction::Restarting)
            .await?;

        let mut args = vec!["restart"];
        if let Some(service) = &service {
            args.push(service);
        }

        let stream = self.cli(&project, args)?;
        return Ok(self.wait(project, service, stream));
    }
}
