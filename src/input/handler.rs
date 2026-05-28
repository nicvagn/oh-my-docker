use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crate::app::event::AppEvent;
use crate::app::mode;
use crate::app::mode::Mode;
use crate::app::state::AppState;

pub fn handle_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    if key.kind != KeyEventKind::Press && key.kind != KeyEventKind::Repeat {
        return None;
    }

    if let Mode::Logs(_) = state.navigation.mode_stack.current() {
        if let Some(ref log) = state.navigation.logs {
            if log.search_active && key.code == KeyCode::Esc {
                return Some(AppEvent::SearchLogs(String::new()));
            }
        }
    }

    if state.error_persistent {
        return Some(AppEvent::Info(String::new()));
    }

    if key.code == KeyCode::Char('U') {
        return Some(AppEvent::CheckUpdate);
    }

    let in_input_mode = state.containers.filter_active
        || state.images.filter_active
        || state.events.filter_active
        || state.containers.show_column_picker
        || state.images.show_column_picker
        || state.networks.show_column_picker
        || state.volumes.show_column_picker
        || state.navigation.logs.as_ref().map(|l| l.search_active).unwrap_or(false)
        || state.navigation.shell_config.is_some()
        || state.navigation.image_run.is_some()
        || state.explorer.host.filter_active
        || state.explorer.container.filter_active;

    if state.error_persistent {
        return Some(AppEvent::Info(String::new()));
    }

    let km = &state.keymap;
    let code = key.code;
    let mods = key.modifiers;

    if km.is_quit(code, mods) && !in_input_mode {
        return Some(AppEvent::Quit);
    }

    if km.is_back(code, mods) && !in_input_mode {
        if state.containers.selection_mode {
            // Let Esc pass through to the container handler which exits selection mode
        } else if *state.navigation.mode_stack.current() == Mode::Help && state.navigation.mode_stack.len() > 1 {
            return Some(AppEvent::Back);
        } else if *state.navigation.mode_stack.current() == Mode::Help {
            // Help tab (not pushed) — return to previous tab
            return Some(AppEvent::Navigate(mode::tab_to_mode(state.previous_tab)));
        } else if state.navigation.mode_stack.len() > 1 {
            return Some(AppEvent::Back);
        } else {
            return None;
        }
    }

    // ? switches to Help tab (base modes) or shows help overlay (sub-views)
    if km.is_help(code, mods) && !in_input_mode {
        if *state.navigation.mode_stack.current() == Mode::Help {
            return None;
        }
        if mode::mode_to_tab(state.navigation.mode_stack.current()).is_some() {
            return Some(AppEvent::Navigate(Mode::Help));
        } else {
            return Some(AppEvent::ShowHelp);
        }
    }

    // Tab/BackTab navigation for base modes
    if !in_input_mode {
        let current_tab = key.code == KeyCode::Tab || key.code == KeyCode::BackTab;
        if current_tab && mode::mode_to_tab(state.navigation.mode_stack.current()).is_some() {
            let next = match (code, mods) {
                (KeyCode::Tab, KeyModifiers::NONE) => (state.selected_tab + 1) % mode::TAB_COUNT,
                (KeyCode::BackTab, _) | (KeyCode::Tab, KeyModifiers::SHIFT) => {
                    (state.selected_tab + mode::TAB_COUNT - 1) % mode::TAB_COUNT
                }
                _ => return None,
            };
            return Some(AppEvent::Navigate(mode::tab_to_mode(next)));
        }
    }

    // Global view navigation shortcuts — only for base tab modes
    if !in_input_mode && mode::mode_to_tab(state.navigation.mode_stack.current()).is_some() {
        let current = state.navigation.mode_stack.current();
        let target = match code {
            KeyCode::Char('c') if *current != Mode::Containers => Some(Mode::Containers),
            KeyCode::Char('i') if *current != Mode::Images => Some(Mode::Images),
            KeyCode::Char('n') if *current != Mode::Networks => Some(Mode::Networks),
            KeyCode::Char('v') if *current != Mode::Volumes => Some(Mode::Volumes),
            KeyCode::Char('e') if *current != Mode::Events => Some(Mode::Events),
            KeyCode::Char('%') if *current != Mode::Statistics => Some(Mode::Statistics),
            _ => None,
        };
        if let Some(mode) = target {
            return Some(AppEvent::Navigate(mode));
        }
    }

    match state.navigation.mode_stack.current() {
        Mode::Containers => crate::app::handlers::container::handle_key_with_clipboard(key, state),
        Mode::ContainerDetails(_) => crate::app::handlers::navigation::handle_details_key(key, state),
        Mode::Logs(_) => crate::app::handlers::log::handle_key(key, state),
        Mode::Images => crate::app::handlers::image::handle_key_with_clipboard(key, state),
        Mode::ImageRun(_) => crate::app::handlers::image::handle_image_run_key(key, state),
        Mode::ShellConfig(_) => crate::app::handlers::shell::handle_shell_config_key(key, state),
        Mode::Shell(_) => crate::app::handlers::shell::handle_shell_key(key),
        Mode::Events => crate::app::handlers::event::handle_key(key, state),
        Mode::Statistics => crate::app::handlers::statistics::handle_key(key, state),
        Mode::Networks => crate::app::handlers::network::handle_key_with_clipboard(key, state),
        Mode::Volumes => crate::app::handlers::volume::handle_key_with_clipboard(key, state),
        Mode::Help => crate::app::handlers::navigation::handle_help_key(key, state),
        Mode::ConfirmDialog { .. } => crate::app::handlers::navigation::handle_confirm_dialog_key(key),
        Mode::Explorer(_) => crate::app::handlers::explorer::handle_key(key, state),
    }
}
