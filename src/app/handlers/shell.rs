use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::event::{AppEvent, ShellConfigField};
use crate::app::state::AppState;

pub fn handle_shell_key(_key: KeyEvent) -> Option<AppEvent> {
    match _key.code {
        KeyCode::Esc => Some(AppEvent::CloseShell),
        _ => None,
    }
}

pub fn handle_shell_config_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    match (key.code, key.modifiers) {
        (KeyCode::Backspace, _) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => state.navigation.shell_config.as_ref().map(|cfg| {
            let (val, field) = match cfg.field_focus {
                0 => (cfg.shell.as_str(), ShellConfigField::Shell),
                1 => (cfg.user.as_str(), ShellConfigField::User),
                _ => (cfg.workdir.as_str(), ShellConfigField::Workdir),
            };
            let new_val: String = val.chars().take(val.chars().count().saturating_sub(1)).collect();
            AppEvent::ShellConfigFieldUpdate(field, new_val)
        }),
        (KeyCode::Esc, _) => Some(AppEvent::Back),
        (KeyCode::Enter, _) => Some(AppEvent::ShellConfigSubmit),
        (KeyCode::Tab, _) | (KeyCode::Down, _) => Some(AppEvent::ShellConfigFocusNext),
        (KeyCode::Up, _) => Some(AppEvent::ShellConfigFocusPrev),
        (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => state.navigation.shell_config.as_ref().map(|cfg| {
            let (val, field) = match cfg.field_focus {
                0 => (cfg.shell.as_str(), ShellConfigField::Shell),
                1 => (cfg.user.as_str(), ShellConfigField::User),
                _ => (cfg.workdir.as_str(), ShellConfigField::Workdir),
            };
            AppEvent::ShellConfigFieldUpdate(field, format!("{}{}", val, c))
        }),
        _ => None,
    }
}
