use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::event::{AppEvent, ConfirmAction};
use crate::app::mode::Mode;
use crate::app::state::AppState;

pub fn handle_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    if state.containers.filter_active {
        match (key.code, key.modifiers) {
            (KeyCode::Backspace, _) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                let new_q = state.containers.filter.chars().take(state.containers.filter.chars().count().saturating_sub(1)).collect();
                Some(AppEvent::FilterContainers(new_q))
            }
            (KeyCode::Esc, _) => Some(AppEvent::FilterContainers(String::new())),
            (KeyCode::Enter, _) => Some(AppEvent::FilterSubmit(None)),
            (KeyCode::Down, _) => {
                let next = (state.containers.selected + 1).min(state.containers.filtered.len().saturating_sub(1));
                Some(AppEvent::FilterSubmit(Some(next)))
            }
            (KeyCode::Up, _) => {
                let prev = state.containers.selected.saturating_sub(1);
                Some(AppEvent::FilterSubmit(Some(prev)))
            }
            (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                let new_q = format!("{}{}", state.containers.filter, c);
                Some(AppEvent::FilterContainers(new_q))
            }
            _ => None,
        }
    } else {
        let km = &state.keymap;
        let code = key.code;
        let mods = key.modifiers;

        if km.is_navigate_down(code, mods) {
            let next = (state.containers.selected + 1).min(state.containers.filtered.len().saturating_sub(1));
            return Some(AppEvent::SelectContainer(next));
        }
        if km.is_navigate_up(code, mods) {
            let prev = state.containers.selected.saturating_sub(1);
            return Some(AppEvent::SelectContainer(prev));
        }
        if km.is_open_details(code, mods) {
            return Some(AppEvent::ShowDetails);
        }
        if km.is_search(code, mods) {
            return Some(AppEvent::ActivateFilter);
        }
        if km.is_open_logs(code, mods) {
            return state.containers.filtered.get(state.containers.selected)
                .and_then(|&idx| state.containers.items.get(idx))
                .map(|c| AppEvent::Navigate(Mode::Logs(c.id.clone())));
        }
        if km.is_open_shell(code, mods) {
            return state.containers.filtered.get(state.containers.selected)
                .and_then(|&idx| state.containers.items.get(idx))
                .map(|c| AppEvent::Navigate(Mode::ShellConfig(c.id.clone())));
        }
        if km.is_restart(code, mods) {
            return state.containers.filtered.get(state.containers.selected)
                .and_then(|&idx| state.containers.items.get(idx))
                .map(|c| AppEvent::RestartContainer(c.id.clone()));
        }
        if km.is_start_stop(code, mods) {
            if state.containers.selection_mode {
                let ids: Vec<String> = state.containers.selected_ids.iter().cloned().collect();
                if ids.is_empty() {
                    return state.containers.filtered.get(state.containers.selected)
                        .and_then(|&idx| state.containers.items.get(idx))
                        .map(|c| if c.state == "running" {
                            AppEvent::StopContainer(c.id.clone())
                        } else {
                            AppEvent::StartContainer(c.id.clone())
                        });
                } else {
                    return Some(AppEvent::BatchToggleContainers(ids));
                }
            } else {
                return state.containers.filtered.get(state.containers.selected)
                    .and_then(|&idx| state.containers.items.get(idx))
                    .map(|c| if c.state == "running" {
                        AppEvent::StopContainer(c.id.clone())
                    } else {
                        AppEvent::StartContainer(c.id.clone())
                    });
            }
        }
        if km.is_delete(code, mods) {
            if state.containers.selection_mode {
                let ids: Vec<String> = state.containers.selected_ids.iter().cloned().collect();
                if ids.is_empty() {
                    return state.containers.filtered.get(state.containers.selected)
                        .and_then(|&idx| state.containers.items.get(idx))
                        .map(|c| AppEvent::ShowConfirmDialog(
                            format!("Delete container '{}'?", c.name),
                            ConfirmAction::DeleteContainer(c.id.clone()),
                        ));
                } else {
                    return Some(AppEvent::ShowConfirmDialog(
                        format!("Delete {} selected container(s)?", ids.len()),
                        ConfirmAction::BatchDeleteContainers,
                    ));
                }
            } else {
                return state.containers.filtered.get(state.containers.selected)
                    .and_then(|&idx| state.containers.items.get(idx))
                    .map(|c| AppEvent::ShowConfirmDialog(
                        format!("Delete container '{}'?", c.name),
                        ConfirmAction::DeleteContainer(c.id.clone()),
                    ));
            }
        }
        if km.is_toggle_selection(code, mods) {
            if !state.containers.selection_mode {
                return Some(AppEvent::ToggleSelectionMode);
            } else {
                return state.containers.filtered.get(state.containers.selected)
                    .and_then(|&idx| state.containers.items.get(idx))
                    .map(|c| AppEvent::ToggleSelectContainer(c.id.clone()));
            }
        }
        if km.is_select_all(code, mods) {
            if state.containers.selection_mode {
                return Some(AppEvent::SelectAllContainers);
            } else {
                return None;
            }
        }
        if km.is_toggle_group_by_project(code, mods) {
            return Some(AppEvent::ToggleGroupByProject);
        }
        if code == KeyCode::Char('i') {
            return Some(AppEvent::Navigate(Mode::Images));
        }
        if code == KeyCode::Char('e') {
            return Some(AppEvent::Navigate(Mode::Events));
        }
        if code == KeyCode::Char('%') {
            return Some(AppEvent::Navigate(Mode::Statistics));
        }
        if code == KeyCode::Char('n') {
            return Some(AppEvent::Navigate(Mode::Networks));
        }
        if code == KeyCode::Char('v') {
            return Some(AppEvent::Navigate(Mode::Volumes));
        }
        if code == KeyCode::Esc {
            if state.containers.selection_mode {
                return Some(AppEvent::ToggleSelectionMode);
            } else {
                return None;
            }
        }
        None
    }
}

pub fn handle_key_with_clipboard(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('y') {
        if let Some(c) = state.containers.filtered.get(state.containers.selected)
            .and_then(|&idx| state.containers.items.get(idx))
        {
            let _ = crate::util::copy_to_clipboard(&c.id);
            return Some(AppEvent::Info(format!("Container ID copied to clipboard")));
        }
    }
    handle_key(key, state)
}
