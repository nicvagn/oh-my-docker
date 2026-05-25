use bollard::Docker;
use bollard::system::EventsOptions;
use anyhow::Result;
use futures_util::StreamExt;
use tokio::sync::mpsc::UnboundedSender;

use crate::app::event::{AppEvent, DockerEvent};

pub async fn stream_events(docker: Docker, tx: UnboundedSender<AppEvent>) -> Result<()> {
    let options = EventsOptions::<String> {
        since: None,
        until: None,
        filters: std::collections::HashMap::new(),
    };

    let mut stream = docker.events(Some(options));
    while let Some(event_result) = stream.next().await {
        match event_result {
            Ok(event) => {
                let kind = event.typ.as_ref().map(|t| t.to_string()).unwrap_or_default();
                let action = event.action.unwrap_or_default();
                let actor = event.actor.as_ref().and_then(|a| a.id.as_deref()).unwrap_or_default().to_string();
                let entry = DockerEvent {
                    timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                    kind,
                    action,
                    actor,
                };
                if tx.send(AppEvent::EventsUpdated(vec![entry])).is_err() {
                    break;
                }
            }
            Err(e) => {
                if tx.send(AppEvent::Info(format!("Events error: {}", e))).is_err() {
                    break;
                }
            }
        }
    }

    Ok(())
}
