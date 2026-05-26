use crate::app::event::{AppEvent, Command};
use crate::app::state::AppState;

pub fn reduce(state: &mut AppState, event: &AppEvent) -> Vec<Command> {
    let mut commands = Vec::new();
    match event {
        AppEvent::LogLines(id, lines) => {
            let should_swap = match state.navigation.logs {
                Some(ref l) => l.container_id != *id,
                None => true,
            };
            if should_swap {
                state.navigation.logs = Some(crate::app::state::LogState {
                    container_id: id.clone(),
                    buffer: Vec::new(),
                    max_lines: 10000,
                    paused: false,
                    search: String::new(),
                    search_active: false,
                    scroll_offset: 0,
                    tail: true,
                    show_timestamps: false,
                    viewport_height: 0,
                });
            }
            if let Some(ref mut log_state) = state.navigation.logs {
                let n = lines.len();
                for line in lines {
                    log_state.buffer.push(line.clone());
                }
                if log_state.paused {
                    log_state.scroll_offset = log_state.scroll_offset.saturating_add(n);
                }
                if log_state.buffer.len() > log_state.max_lines {
                    let excess = log_state.buffer.len() - log_state.max_lines;
                    log_state.buffer.drain(0..excess);
                }
                if log_state.tail {
                    log_state.scroll_offset = 0;
                }
            }
        }
        AppEvent::TogglePause => {
            if let Some(ref mut log) = state.navigation.logs {
                log.paused = !log.paused;
                if log.paused {
                    log.tail = false;
                } else {
                    log.tail = true;
                    log.scroll_offset = 0;
                }
            }
        }
        AppEvent::ActivateLogSearch => {
            if let Some(ref mut log) = state.navigation.logs {
                log.search_active = true;
            }
        }
        AppEvent::SearchLogs(q) => {
            if let Some(ref mut log) = state.navigation.logs {
                log.search = q.clone();
                log.search_active = !q.is_empty();
            }
        }
        AppEvent::SubmitLogSearch => {
            if let Some(ref mut log) = state.navigation.logs {
                log.search_active = false;
            }
        }
        AppEvent::ScrollLogs(delta) => {
            if let Some(ref mut log) = state.navigation.logs {
                let max_offset = log.buffer.len().saturating_sub(log.viewport_height);
                log.scroll_offset = crate::util::scroll_offset(log.scroll_offset, *delta, max_offset);
                log.tail = log.scroll_offset == 0;
            }
        }
        AppEvent::ToggleLogTimestamps => {
            if let Some(ref mut log) = state.navigation.logs {
                log.show_timestamps = !log.show_timestamps;
            }
        }
        AppEvent::ExportLogs(container_id) => {
            let buffer: Vec<String> = state.navigation.logs.as_ref()
                .map(|l| l.buffer.iter().map(|e| e.message.clone()).collect())
                .unwrap_or_default();
            if buffer.is_empty() {
                state.error = Some("No logs to export".to_string());
                state.error_timer = 5;
            } else {
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let filename = format!("/tmp/omdocker_logs_{}_{}.log", container_id, ts);
                let fname = filename.clone();
                let lines = buffer.clone();
                commands.push(Command::ExportLogs(fname, lines));
            }
        }
        _ => {}
    }
    commands
}
