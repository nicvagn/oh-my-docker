use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::event::AppEvent;
use crate::app::state::AppState;

pub fn handle_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    let search_active = state.navigation.logs.as_ref().map(|l| l.search_active).unwrap_or(false);
    if search_active {
        match (key.code, key.modifiers) {
            (KeyCode::Backspace, _) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                state.navigation.logs.as_ref().map(|l| {
                    let new_q = l.search.chars().take(l.search.chars().count().saturating_sub(1)).collect();
                    AppEvent::SearchLogs(new_q)
                })
            }
            (KeyCode::Esc, _) => Some(AppEvent::SearchLogs(String::new())),
            (KeyCode::Enter, _) => Some(AppEvent::SubmitLogSearch),
            (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                state.navigation.logs.as_ref().map(|l| {
                    let new_q = format!("{}{}", l.search, c);
                    AppEvent::SearchLogs(new_q)
                })
            }
            _ => None,
        }
    } else {
        match key.code {
            KeyCode::Char(' ') | KeyCode::Char('p') => Some(AppEvent::TogglePause),
            KeyCode::Char('r') => {
                state.navigation.logs.as_ref().and_then(|l| {
                    if l.paused { Some(AppEvent::TogglePause) } else { None }
                })
            }
            KeyCode::Char('/') => Some(AppEvent::ActivateLogSearch),
            KeyCode::Char('g') => Some(AppEvent::JumpTop),
            KeyCode::Char('G') => Some(AppEvent::JumpBottom),
            KeyCode::Char('s') => state.navigation.logs.as_ref().map(|l| AppEvent::ExportLogs(l.container_id.clone())),
            KeyCode::Char('T') => Some(AppEvent::ToggleLogTimestamps),
            KeyCode::Up | KeyCode::Char('k') => Some(AppEvent::ScrollLogs(1)),
            KeyCode::Down | KeyCode::Char('j') => Some(AppEvent::ScrollLogs(-1)),
            KeyCode::PageUp => Some(AppEvent::ScrollLogs(20)),
            KeyCode::PageDown => Some(AppEvent::ScrollLogs(-20)),
            _ => None,
        }
    }
}
