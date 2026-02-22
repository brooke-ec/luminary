//! This module implements a stream for listening to changes from Docker engine.

use std::{
    collections::HashMap,
    pin::Pin,
    task::{Context, Poll, ready},
};

use bollard::{models::EventMessage, query_parameters::EventsOptionsBuilder};
use eyre::Result;
use futures_util::{FutureExt, Stream, future::BoxFuture};
use luminary_library::StreamExt;
use pin_project::pin_project;

use crate::{
    LuminaryEngine,
    model::{LuminaryProjectList, LuminaryStatus},
    project::{LuminaryProjectListExt, LuminaryProjectParser},
};

impl LuminaryEngine {
    /// Subscribes to Docker events and emits an updated list of Luminary projects whenever a change occurs.
    pub fn subscribe<'a>(&'a self) -> impl Stream<Item = LuminaryProjectList> + use<'a> {
        let mut filters = HashMap::new();
        filters.insert("type", vec!["container"]);

        let options = EventsOptionsBuilder::default().filters(&filters).build();
        return self
            .docker
            .events(Some(options))
            .then_fold(LuminaryProjectList::new(), move |state, event| {
                process_event(self, state, event)
            });
    }
}

/// Process a Docker event and update the project list accordingly.
async fn process_event<'a>(
    engine: &'a LuminaryEngine,
    mut state: LuminaryProjectList,
    e: Result<EventMessage, bollard::errors::Error>,
) -> (LuminaryProjectList, LuminaryProjectList) {
    if let Ok(event) = e {
        if let Some(action) = event.action
            && let Some(actor) = event.actor
            && let Some(labels) = actor.attributes
            && let Some(status) = parse_action(action.clone())
            && let Some(project) = engine.parse_labels(status, labels)
        {
            state.merge_project(project);
        }
    }

    return (state, state.clone());
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
