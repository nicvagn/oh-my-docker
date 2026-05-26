use crossterm::event::{KeyCode, KeyEvent};
use crate::app::event::AppEvent;

pub fn handle_key(key: KeyEvent) -> Option<AppEvent> {
    match key.code {
        KeyCode::Esc => Some(AppEvent::Back),
        KeyCode::Char('t') => Some(AppEvent::ToggleSortDirection),
        KeyCode::Left => Some(AppEvent::CycleSortStat(-1)),
        KeyCode::Right => Some(AppEvent::CycleSortStat(1)),
        _ => None,
    }
}
