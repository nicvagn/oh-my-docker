use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::event::AppEvent;
use crate::app::state::AppState;

pub fn handle_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    if state.events.filter_active {
        match (key.code, key.modifiers) {
            (KeyCode::Backspace, _) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                let new_q = state.events.filter.chars().take(state.events.filter.chars().count().saturating_sub(1)).collect();
                Some(AppEvent::FilterEvents(new_q))
            }
            (KeyCode::Esc, _) => Some(AppEvent::FilterEvents(String::new())),
            (KeyCode::Enter, _) => Some(AppEvent::EventsFilterSubmit),
            (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                let new_q = format!("{}{}", state.events.filter, c);
                Some(AppEvent::FilterEvents(new_q))
            }
            _ => None,
        }
    } else {
        match key.code {
            KeyCode::Char('e') => Some(AppEvent::ExportEvents),
            KeyCode::Char('g') => Some(AppEvent::JumpTop),
            KeyCode::Char('G') => Some(AppEvent::JumpBottom),
            KeyCode::Char('k') | KeyCode::Up => Some(AppEvent::ScrollEvents(1)),
            KeyCode::Char('j') | KeyCode::Down => Some(AppEvent::ScrollEvents(-1)),
            KeyCode::PageUp => Some(AppEvent::ScrollEvents(20)),
            KeyCode::PageDown => Some(AppEvent::ScrollEvents(-20)),
            KeyCode::Char('/') => Some(AppEvent::ActivateEventsFilter),
            _ => None,
        }
    }
}
