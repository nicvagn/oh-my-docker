use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::event::{AppEvent, ConfirmAction};
use crate::app::mode::Mode;
use crate::app::state::AppState;
use crate::ui::resource_panel::{VolumeResource, Resource};

pub fn handle_key_with_clipboard(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('o') {
        return Some(AppEvent::ToggleColumnPicker);
    }
    handle_key(key, state)
}

pub fn handle_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    if state.volumes.show_column_picker {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => Some(AppEvent::ToggleColumnPicker),
            (KeyCode::Char(' '), _) | (KeyCode::Enter, _) => {
                let names = ["name", "driver", "size", "mountpoint"];
                let idx = state.volumes.column_picker_selection.min(names.len() - 1);
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
    } else {
        let km = &state.keymap;
        let code = key.code;
        let mods = key.modifiers;

        if code == KeyCode::Esc {
            if state.volumes.selection_mode {
                return Some(AppEvent::ToggleSelectionMode);
            }
            return Some(AppEvent::Back);
        }
        if km.is_navigate_down(code, mods) || code == KeyCode::Down {
            let next = (state.volumes.selected + 1).min(state.volumes.items.len().saturating_sub(1));
            return Some(AppEvent::SelectVolume(next));
        }
        if km.is_navigate_up(code, mods) || code == KeyCode::Up {
            let prev = state.volumes.selected.saturating_sub(1);
            return Some(AppEvent::SelectVolume(prev));
        }
        if km.is_sort_direction(code, mods) {
            return Some(AppEvent::ToggleSortDirection);
        }
        if code == KeyCode::Left {
            let n = VolumeResource::column_headers().len();
            let next = ((state.volumes.sort_column as i32 - 1).rem_euclid(n as i32)) as usize;
            return Some(AppEvent::SortByColumn(next));
        }
        if code == KeyCode::Right {
            let n = VolumeResource::column_headers().len();
            let next = ((state.volumes.sort_column as i32 + 1).rem_euclid(n as i32)) as usize;
            return Some(AppEvent::SortByColumn(next));
        }
        if km.is_toggle_selection(code, mods) {
            if !state.volumes.selection_mode {
                return Some(AppEvent::ToggleSelectionMode);
            } else {
                return state.volumes.filtered.get(state.volumes.selected)
                    .and_then(|&idx| state.volumes.items.get(idx))
                    .map(|v| AppEvent::ToggleSelectResource(v.name.clone()));
            }
        }
        if km.is_select_all(code, mods) {
            if state.volumes.selection_mode {
                return Some(AppEvent::SelectAllResources);
            } else {
                return None;
            }
        }
        if km.is_delete(code, mods) {
            if state.volumes.selection_mode {
                let ids: Vec<String> = state.volumes.selected_ids.iter().cloned().collect();
                if ids.is_empty() {
                    return state.volumes.filtered.get(state.volumes.selected)
                        .and_then(|&idx| state.volumes.items.get(idx))
                        .map(|v| AppEvent::ShowConfirmDialog(
                            format!("Remove volume '{}'?", v.name),
                            ConfirmAction::RemoveVolume(v.name.clone()),
                        ));
                } else {
                    return Some(AppEvent::ShowConfirmDialog(
                        format!("Remove {} selected volume(s)?", ids.len()),
                        ConfirmAction::BatchDeleteVolumes,
                    ));
                }
            } else {
                return state.volumes.filtered.get(state.volumes.selected)
                    .and_then(|&idx| state.volumes.items.get(idx))
                    .map(|v| AppEvent::ShowConfirmDialog(
                        format!("Remove volume '{}'?", v.name),
                        ConfirmAction::RemoveVolume(v.name.clone()),
                    ));
            }
        }
        if code == KeyCode::Char('x') && mods == KeyModifiers::NONE {
            return state.volumes.filtered.get(state.volumes.selected)
                .and_then(|&idx| state.volumes.items.get(idx))
                .map(|v| AppEvent::Navigate(Mode::ExplorerVolume(v.mountpoint.clone(), v.name.clone())));
        }
        None
    }
}
