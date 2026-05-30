use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::event::{AppEvent, ConfirmAction};
use crate::app::mode::Mode;
use crate::app::state::AppState;

pub fn handle_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    if state.containers.show_column_picker {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => Some(AppEvent::ToggleColumnPicker),
            (KeyCode::Char(' '), _) | (KeyCode::Enter, _) => {
                let names = ["name", "image", "state", "ports"];
                let idx = state.containers.column_picker_selection.min(names.len() - 1);
                Some(AppEvent::ToggleColumn(names[idx].to_string()))
            }
            _ if state.keymap.is_navigate_down(key.code, key.modifiers) || key.code == KeyCode::Down => {
                Some(AppEvent::ToggleColumn("next".to_string()))
            }
            _ if state.keymap.is_navigate_up(key.code, key.modifiers) || key.code == KeyCode::Up => {
                Some(AppEvent::ToggleColumn("prev".to_string()))
            }
            _ => None,
        }
    } else if state.containers.filter_active {
        use crate::app::handlers::FilterAction;
        match crate::app::handlers::handle_filter_input(key, &state.containers.filter, state.containers.selected, state.containers.filtered.len()) {
            Some(FilterAction::Update(q)) => Some(AppEvent::FilterContainers(q)),
            Some(FilterAction::Clear) => Some(AppEvent::FilterContainers(String::new())),
            Some(FilterAction::Submit(idx)) => Some(AppEvent::FilterSubmit(idx)),
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
        if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('u') {
            let page = 20;
            let prev = state.containers.selected.saturating_sub(page);
            return Some(AppEvent::SelectContainer(prev));
        }
        if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('d') {
            let page = 20;
            let next = (state.containers.selected + page).min(state.containers.filtered.len().saturating_sub(1));
            return Some(AppEvent::SelectContainer(next));
        }
        if km.is_jump_top(code, mods) {
            return Some(AppEvent::SelectContainer(0));
        }
        if km.is_jump_bottom(code, mods) {
            let last = state.containers.filtered.len().saturating_sub(1);
            return Some(AppEvent::SelectContainer(last));
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
        if code == KeyCode::Char('x') && mods == KeyModifiers::NONE {
            return state.containers.filtered.get(state.containers.selected)
                .and_then(|&idx| state.containers.items.get(idx))
                .map(|c| AppEvent::Navigate(Mode::Explorer(c.id.clone())));
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
        if code == KeyCode::Char('S') && (mods == KeyModifiers::NONE || mods == KeyModifiers::SHIFT) {
            return Some(AppEvent::CycleStatusFilter);
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
            return crate::app::handlers::clipboard_copy(&c.id);
        }
    }
    if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('o') {
        return Some(AppEvent::ToggleColumnPicker);
    }
    handle_key(key, state)
}
