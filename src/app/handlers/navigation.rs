use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::event::AppEvent;
use crate::app::mode::Mode;
use crate::app::state::AppState;

pub fn handle_details_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    let km = &state.keymap;
    let code = key.code;
    let mods = key.modifiers;

    if km.is_open_logs(code, mods) {
        return state.navigation.details.as_ref().map(|d| AppEvent::Navigate(Mode::Logs(d.id.clone())));
    }
    if km.is_open_shell(code, mods) {
        return state.navigation.details.as_ref().map(|d| AppEvent::Navigate(Mode::ShellConfig(d.id.clone())));
    }
    if km.is_restart(code, mods) {
        return state.navigation.details.as_ref().map(|d| AppEvent::RestartContainer(d.id.clone()));
    }
    if km.is_start_stop(code, mods) {
        return state.navigation.details.as_ref().map(|d| {
            let cid = d.id.clone();
            let container = state.containers.items.iter().find(|c| c.id == d.id);
            match container.map(|c| c.state.as_str()) {
                Some("running") => AppEvent::StopContainer(cid),
                _ => AppEvent::StartContainer(cid),
            }
        });
    }
    if code == KeyCode::Char('x') && mods == KeyModifiers::NONE {
        return state.navigation.details.as_ref().map(|d| AppEvent::Navigate(Mode::Explorer(d.id.clone())));
    }
    if km.is_navigate_up(code, mods) || code == KeyCode::Up {
        return Some(AppEvent::ScrollDetails(-1));
    }
    if km.is_navigate_down(code, mods) || code == KeyCode::Down {
        return Some(AppEvent::ScrollDetails(1));
    }
    if code == KeyCode::PageUp {
        return Some(AppEvent::ScrollDetails(-20));
    }
    if code == KeyCode::PageDown {
        return Some(AppEvent::ScrollDetails(20));
    }
    if km.is_jump_top(code, mods) {
        return Some(AppEvent::ScrollDetails(10000));
    }
    if km.is_jump_bottom(code, mods) {
        return Some(AppEvent::ScrollDetails(-10000));
    }
    None
}

pub fn handle_confirm_dialog_key(key: KeyEvent) -> Option<AppEvent> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => Some(AppEvent::ConfirmYes),
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Some(AppEvent::ConfirmNo),
        _ => None,
    }
}

pub fn handle_help_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    let km = &state.keymap;
    let code = key.code;
    let mods = key.modifiers;

    if code == KeyCode::Esc || km.is_help(code, mods) {
        return Some(AppEvent::HideHelp);
    }
    if km.is_navigate_down(code, mods) || code == KeyCode::Down || code == KeyCode::PageDown {
        return Some(AppEvent::ScrollHelp(1));
    }
    if km.is_navigate_up(code, mods) || code == KeyCode::Up || code == KeyCode::PageUp {
        return Some(AppEvent::ScrollHelp(-1));
    }
    if km.is_jump_top(code, mods) {
        return Some(AppEvent::ScrollHelp(-10000));
    }
    if km.is_jump_bottom(code, mods) {
        return Some(AppEvent::ScrollHelp(10000));
    }
    None
}
