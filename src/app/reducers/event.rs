use std::time::Instant;

use crate::app::event::{AppEvent, Command};
use crate::app::state::AppState;

pub fn reduce(state: &mut AppState, event: &AppEvent) -> Vec<Command> {
    let mut commands = Vec::new();
    match event {
        AppEvent::EventsUpdated(events) => {
            for e in events {
                state.events.buffer.push_back(e.clone());
            }
            if state.events.buffer.len() > state.events.max_events {
                state.events.buffer.truncate(state.events.max_events);
            }
            state.events.last_updated = Some(Instant::now());
        }
        AppEvent::ActivateEventsFilter => {
            state.events.filter_active = true;
        }
        AppEvent::EventsFilterSubmit => {
            state.events.filter_active = false;
        }
        AppEvent::FilterEvents(q) => {
            state.events.filter = q.clone();
            if state.events.filter.is_empty() {
                state.events.filter_active = false;
            }
        }
        AppEvent::ExportEvents => {
            let buffer: Vec<String> = state.events.buffer.iter().map(|e| {
                format!("{} {} {} {}", e.timestamp, e.kind, e.action, e.actor)
            }).collect();
            if buffer.is_empty() {
                state.error = Some("No events to export".to_string());
                state.error_timer = 5;
            } else {
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let filename = format!("/tmp/omdocker_events_{}.log", ts);
                let fname = filename.clone();
                let lines = buffer.clone();
                commands.push(Command::ExportLogs(fname, lines));
            }
        }
        AppEvent::ScrollEvents(delta) => {
            let max_offset = state.events.buffer.len().saturating_sub(state.events.viewport_height);
            state.events.scroll_offset = crate::util::scroll_offset(state.events.scroll_offset, *delta, max_offset);
        }
        _ => {}
    }
    commands
}
