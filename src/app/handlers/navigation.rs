use crossterm::event::{KeyCode, KeyEvent};
use crate::app::event::AppEvent;
use crate::app::mode::Mode;
use crate::app::state::AppState;

pub fn handle_details_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    match key.code {
        KeyCode::Char('l') => state.navigation.details.as_ref().map(|d| AppEvent::Navigate(Mode::Logs(d.id.clone()))),
        KeyCode::Char('s') => state.navigation.details.as_ref().map(|d| AppEvent::Navigate(Mode::ShellConfig(d.id.clone()))),
        KeyCode::Char('r') => state.navigation.details.as_ref().map(|d| AppEvent::RestartContainer(d.id.clone())),
        KeyCode::Char('S') => state.navigation.details.as_ref().map(|d| {
            let cid = d.id.clone();
            let container = state.containers.items.iter().find(|c| c.id == d.id);
            match container.map(|c| c.state.as_str()) {
                Some("running") => AppEvent::StopContainer(cid),
                _ => AppEvent::StartContainer(cid),
            }
        }),
        KeyCode::Up | KeyCode::Char('k') => Some(AppEvent::ScrollDetails(-1)),
        KeyCode::Down | KeyCode::Char('j') => Some(AppEvent::ScrollDetails(1)),
        KeyCode::PageUp => Some(AppEvent::ScrollDetails(-20)),
        KeyCode::PageDown => Some(AppEvent::ScrollDetails(20)),
        KeyCode::Char('g') => Some(AppEvent::ScrollDetails(10000)),
        KeyCode::Char('G') => Some(AppEvent::ScrollDetails(-10000)),
        _ => None,
    }
}

pub fn handle_confirm_dialog_key(key: KeyEvent) -> Option<AppEvent> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => Some(AppEvent::ConfirmYes),
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Some(AppEvent::ConfirmNo),
        _ => None,
    }
}

pub fn handle_help_key(key: KeyEvent) -> Option<AppEvent> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('?') => Some(AppEvent::HideHelp),
        KeyCode::Char('j') | KeyCode::Down | KeyCode::PageDown => Some(AppEvent::ScrollHelp(1)),
        KeyCode::Char('k') | KeyCode::Up | KeyCode::PageUp => Some(AppEvent::ScrollHelp(-1)),
        KeyCode::Char('g') => Some(AppEvent::ScrollHelp(10000)),
        KeyCode::Char('G') => Some(AppEvent::ScrollHelp(-10000)),
        _ => None,
    }
}
