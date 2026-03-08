use std::collections::HashMap;

use eyre::{Result, bail};
use futures_util::{Stream, StreamExt};

use crate::core::{LuminaryAction, LuminaryEngine, LuminaryProject, LuminaryStatus};

impl LuminaryEngine {
    async fn set_action(&self, project: &str, action: LuminaryAction) -> bool {
        let mut actions = self.actions.write().await;

        let _ = self.actions_channel.send(LuminaryProject {
            name: project.to_string(),
            status: LuminaryStatus::Down,
            action,
            services: HashMap::new(),
        });

        return match action {
            LuminaryAction::Idle => actions.remove(project).is_some(),
            _ => actions.insert(project.to_string(), action.clone()).is_some(),
        };
    }

    pub(super) async fn get_action(&self, project: &str) -> LuminaryAction {
        self.actions
            .read()
            .await
            .get(project)
            .cloned()
            .unwrap_or(LuminaryAction::Idle)
    }

    fn wait<T>(
        &self,
        project: String,
        mut stream: impl Stream<Item = T> + Unpin + Send + 'static,
    ) -> tokio::task::JoinHandle<()> {
        let this = self.clone();
        return tokio::spawn(async move {
            while let Some(_) = stream.next().await {}
            this.set_action(&project, LuminaryAction::Idle).await;
        });
    }

    pub async fn restart(
        &self,
        project: String,
        service: Option<&str>,
    ) -> Result<tokio::task::JoinHandle<()>> {
        if self.set_action(&project, LuminaryAction::Restarting).await {
            bail!("Project '{}' is already processing an action", project);
        }

        let mut args = vec!["restart"];
        if let Some(service) = service {
            args.push(service);
        }

        let stream = self.cli(&project, args)?;
        return Ok(self.wait(project, stream));
    }
}
