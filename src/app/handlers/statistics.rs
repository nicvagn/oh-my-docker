use crossterm::event::{KeyCode, KeyEvent};
use crate::app::event::AppEvent;
use crate::app::state::AppState;

pub fn handle_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    let km = &state.keymap;
    let code = key.code;
    let mods = key.modifiers;

    if code == KeyCode::Esc {
        return Some(AppEvent::Back);
    }
    if km.is_jump_top(code, mods) {
        return Some(AppEvent::ScrollStatistics(i32::MAX));
    }
    if km.is_jump_bottom(code, mods) {
        return Some(AppEvent::ScrollStatistics(i32::MIN));
    }
    if km.is_navigate_up(code, mods) || code == KeyCode::Up {
        return Some(AppEvent::ScrollStatistics(-1));
    }
    if km.is_navigate_down(code, mods) || code == KeyCode::Down {
        return Some(AppEvent::ScrollStatistics(1));
    }
    if code == KeyCode::PageUp {
        return Some(AppEvent::ScrollStatistics(-20));
    }
    if code == KeyCode::PageDown {
        return Some(AppEvent::ScrollStatistics(20));
    }
    if km.is_sort_direction(code, mods) {
        return Some(AppEvent::ToggleSortDirection);
    }
    if code == KeyCode::Left {
        return Some(AppEvent::CycleSortStat(-1));
    }
    if code == KeyCode::Right {
        return Some(AppEvent::CycleSortStat(1));
    }
    None
}
