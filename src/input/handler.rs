use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use crate::app::event::AppEvent;
use crate::app::mode::Mode;
use crate::app::state::AppState;
use crate::input::keys;

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
        || state.navigation.logs.as_ref().map(|l| l.search_active).unwrap_or(false)
        || state.navigation.shell_config.is_some()
        || state.navigation.image_run.is_some();

    if key.code != KeyCode::Char('q') || !in_input_mode {
        if let Some(action) = keys::global_action(key.code) {
            match action {
                keys::Action::Quit => return Some(AppEvent::Quit),
                keys::Action::Back => {
                    if state.containers.selection_mode {
                    } else if *state.navigation.mode_stack.current() == Mode::Help {
                        return Some(AppEvent::HideHelp);
                    } else if state.navigation.mode_stack.len() > 1 {
                        return Some(AppEvent::Back);
                    } else {
                        return None;
                    }
                }
                keys::Action::ShowHelp => return Some(AppEvent::ShowHelp),
            }
        }
    }

    match state.navigation.mode_stack.current() {
        Mode::Containers => crate::app::handlers::container::handle_key(key, state),
        Mode::ContainerDetails(_) => crate::app::handlers::navigation::handle_details_key(key, state),
        Mode::Logs(_) => crate::app::handlers::log::handle_key(key, state),
        Mode::Images => crate::app::handlers::image::handle_key(key, state),
        Mode::ImageRun(_) => crate::app::handlers::image::handle_image_run_key(key, state),
        Mode::ShellConfig(_) => crate::app::handlers::shell::handle_shell_config_key(key, state),
        Mode::Shell(_) => crate::app::handlers::shell::handle_shell_key(key),
        Mode::Events => crate::app::handlers::event::handle_key(key, state),
        Mode::Statistics => crate::app::handlers::statistics::handle_key(key),
        Mode::Networks => crate::app::handlers::network::handle_key(key, state),
        Mode::Volumes => crate::app::handlers::volume::handle_key(key, state),
        Mode::Help => crate::app::handlers::navigation::handle_help_key(key),
        Mode::ConfirmDialog { .. } => crate::app::handlers::navigation::handle_confirm_dialog_key(key),
    }
}
