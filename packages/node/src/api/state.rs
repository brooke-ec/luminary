use std::convert::Infallible;

use async_stream::stream;
use axum::{
    extract::State,
    response::{Sse, sse::Event},
};
use futures_util::{Stream, StreamExt};
use serde_json::json;

use crate::state::LuminaryState;

impl LuminaryState {
    pub fn spawn_worker(&self) {
        let channel = self.channel.clone();
        let engine = self.engine.clone();

        tokio::spawn(async move {
            let mut reciever = engine.stream();

            while let Some(result) = reciever.next().await {
                let project = result.unwrap();
                channel.send(project).unwrap();
            }
        });
    }
}

pub async fn handle_request(
    state: State<LuminaryState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.channel.subscribe();
    let mut current = json!({});

    return Sse::new(stream! {
        let mut new = state.engine.list_projects().await.unwrap();

        loop {
            let value = serde_json::to_value(&new).unwrap();
            let diff = json_patch::diff(&current, &value);
            current = value;

            yield Ok(Event::default().json_data(diff).unwrap());

            if let Ok(project) = rx.recv().await {
                project.merge_into(&mut new);
            }
        }
    });
}
