//! This module implements a stream for listening to changes from Docker engine.

use std::collections::HashMap;

use bollard::query_parameters::EventsOptionsBuilder;
use eyre::{Context, Result};
use futures_util::{Stream, StreamExt};

use crate::{
    LuminaryEngine,
    model::{LuminaryProject, LuminaryStatus},
    project::LuminaryProjectParser,
};

impl LuminaryEngine {
    /// Subscribes to Docker events and emits a partial `LuminaryProject` whenever one of its containers changes state.
    pub fn subscribe(&self) -> impl Stream<Item = Result<LuminaryProject>> {
        let mut filters = HashMap::new();
        filters.insert("type", vec!["container"]);

        let options = EventsOptionsBuilder::default().filters(&filters).build();
        self.docker.events(Some(options)).filter_map(move |event| async {
            match event.wrap_err("Failed to receive Docker event") {
                Err(err) => Some(Err(err)),
                Ok(event) => {
                    if let Some(actor) = event.actor
                        && let Some(action) = event.action
                        && let Some(labels) = actor.attributes
                        && let Some(status) = parse_action(action.clone())
                        && let Some(project) = self.parse_labels(status, labels)
                    {
                        return Some(Ok(project));
                    } else {
                        return None;
                    }
                }
            }
        })
    }
}

/// Translate a Docker event action into a LuminaryStatus, if possible.
fn parse_action(action: String) -> Option<LuminaryStatus> {
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
