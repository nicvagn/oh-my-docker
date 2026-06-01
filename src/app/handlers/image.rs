use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::event::{AppEvent, ConfirmAction, ImageRunField};
use crate::app::state::AppState;
use crate::ui::resource_panel::{ImageResource, Resource};

pub fn handle_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    if state.images.show_column_picker {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => Some(AppEvent::ToggleColumnPicker),
            (KeyCode::Char(' '), _) | (KeyCode::Enter, _) => {
                let names = ["repository", "tag", "id", "size"];
                let idx = state.images.column_picker_selection.min(names.len() - 1);
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
    } else if state.images.filter_active {
        use crate::app::handlers::FilterAction;
        match crate::app::handlers::handle_filter_input(key, &state.images.filter, state.images.selected, state.images.filtered.len()) {
            Some(FilterAction::Update(q)) => Some(AppEvent::FilterImages(q)),
            Some(FilterAction::Clear) => Some(AppEvent::FilterImages(String::new())),
            Some(FilterAction::Submit(idx)) => Some(AppEvent::FilterSubmit(idx)),
            _ => None,
        }
    } else {
        let km = &state.keymap;
        let code = key.code;
        let mods = key.modifiers;

        if km.is_navigate_images(code, mods) || code == KeyCode::Down {
            let next = (state.images.selected + 1).min(state.images.filtered.len().saturating_sub(1));
            return Some(AppEvent::SelectImage(next));
        }
        if km.is_navigate_up(code, mods) || code == KeyCode::Up {
            let prev = state.images.selected.saturating_sub(1);
            return Some(AppEvent::SelectImage(prev));
        }
        if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('u') {
            let page = 20;
            let prev = state.images.selected.saturating_sub(page);
            return Some(AppEvent::SelectImage(prev));
        }
        if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('d') {
            let page = 20;
            let next = (state.images.selected + page).min(state.images.filtered.len().saturating_sub(1));
            return Some(AppEvent::SelectImage(next));
        }
        if km.is_jump_top(code, mods) {
            return Some(AppEvent::SelectImage(0));
        }
        if km.is_jump_bottom(code, mods) {
            let last = state.images.filtered.len().saturating_sub(1);
            return Some(AppEvent::SelectImage(last));
        }
        if km.is_run_image(code, mods) {
            return state.images.filtered.get(state.images.selected)
                .and_then(|&idx| state.images.items.get(idx))
                .map(|img| AppEvent::RunImage(img.repository.clone(), img.tag.clone()));
        }
        if km.is_remove_image(code, mods) {
            if state.images.selection_mode {
                let ids: Vec<String> = state.images.selected_ids.iter().cloned().collect();
                if ids.is_empty() {
                    return state.images.filtered.get(state.images.selected)
                        .and_then(|&idx| state.images.items.get(idx))
                        .map(|img| AppEvent::ShowConfirmDialog(
                            format!("Remove image '{}:{}'?", img.repository, img.tag),
                            ConfirmAction::RemoveImage(img.id.clone()),
                        ));
                } else {
                    return Some(AppEvent::ShowConfirmDialog(
                        format!("Remove {} selected image(s)?", ids.len()),
                        ConfirmAction::BatchDeleteImages,
                    ));
                }
            } else {
                return state.images.filtered.get(state.images.selected)
                    .and_then(|&idx| state.images.items.get(idx))
                    .map(|img| AppEvent::ShowConfirmDialog(
                        format!("Remove image '{}:{}'?", img.repository, img.tag),
                        ConfirmAction::RemoveImage(img.id.clone()),
                    ));
            }
        }
        if km.is_remove_dangling_images(code, mods) {
            return Some(AppEvent::ShowConfirmDialog(
                "Remove all dangling (<none>) images?".to_string(),
                ConfirmAction::RemoveDanglingImages,
            ));
        }
        if km.is_prune_images(code, mods) {
            return Some(AppEvent::ShowConfirmDialog(
                "Prune all unused images?".to_string(),
                ConfirmAction::PruneUnusedImages,
            ));
        }
        if km.is_sort_direction(code, mods) {
            return Some(AppEvent::ToggleSortDirection);
        }
        if code == KeyCode::Left {
            let n = ImageResource::column_headers().len();
            let next = ((state.images.sort_column as i32 - 1).rem_euclid(n as i32)) as usize;
            return Some(AppEvent::SortByColumn(next));
        }
        if code == KeyCode::Right {
            let n = ImageResource::column_headers().len();
            let next = ((state.images.sort_column as i32 + 1).rem_euclid(n as i32)) as usize;
            return Some(AppEvent::SortByColumn(next));
        }
        if km.is_toggle_selection(code, mods) {
            if !state.images.selection_mode {
                return Some(AppEvent::ToggleSelectionMode);
            } else {
                return state.images.filtered.get(state.images.selected)
                    .and_then(|&idx| state.images.items.get(idx))
                    .map(|img| AppEvent::ToggleSelectResource(img.id.clone()));
            }
        }
        if km.is_select_all(code, mods) {
            if state.images.selection_mode {
                return Some(AppEvent::SelectAllResources);
            } else {
                return None;
            }
        }
        if km.is_search(code, mods) {
            return Some(AppEvent::ActivateImageFilter);
        }
        if code == KeyCode::Esc {
            if state.images.selection_mode {
                return Some(AppEvent::ToggleSelectionMode);
            }
            return None;
        }
        None
    }
}

pub fn handle_key_with_clipboard(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('o') {
        return Some(AppEvent::ToggleColumnPicker);
    }
    handle_key(key, state)
}

pub fn handle_image_run_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    match (key.code, key.modifiers) {
        (KeyCode::Backspace, _) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => state.navigation.image_run.as_ref().and_then(|run| {
            let is_text = matches!(run.field_focus, 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 9 | 10 | 11 | 12 | 13);
            if !is_text { return None; }
            let (val, field) = match run.field_focus {
                0 => (run.command.as_str(), ImageRunField::Command),
                1 => (run.shell.as_str(), ImageRunField::Shell),
                2 => (run.user.as_str(), ImageRunField::User),
                3 => (run.workdir.as_str(), ImageRunField::Workdir),
                4 => (run.env_vars.as_str(), ImageRunField::EnvVars),
                5 => (run.port_mapping.as_str(), ImageRunField::PortMapping),
                6 => (run.volumes.as_str(), ImageRunField::Volumes),
                7 => (run.container_name.as_str(), ImageRunField::ContainerName),
                9 => (run.restart_policy.as_str(), ImageRunField::RestartPolicy),
                10 => (run.memory_limit.as_str(), ImageRunField::MemoryLimit),
                11 => (run.cpu_limit.as_str(), ImageRunField::CpuLimit),
                12 => (run.network.as_str(), ImageRunField::Network),
                13 => (run.labels.as_str(), ImageRunField::Labels),
                _ => return None,
            };
            let new_val: String = val.chars().take(val.chars().count().saturating_sub(1)).collect();
            Some(AppEvent::ImageRunFieldUpdate(field, new_val))
        }),
        (KeyCode::Esc, _) => Some(AppEvent::Back),
        (KeyCode::Enter, _) => Some(AppEvent::ImageRunSubmit),
        (KeyCode::Tab, _) | (KeyCode::Down, _) => Some(AppEvent::ImageRunFocusNext),
        (KeyCode::Up, _) => Some(AppEvent::ImageRunFocusPrev),
        (KeyCode::Char('a'), KeyModifiers::NONE) | (KeyCode::Char('a'), KeyModifiers::SHIFT) => {
            state.navigation.image_run.as_ref().and_then(|run| {
                match run.field_focus {
                    8 => Some(AppEvent::ImageRunToggleAutoremove),
                    9 => {
                        let next = match run.restart_policy.as_str() {
                            "" => "always",
                            "always" => "on-failure",
                            "on-failure" => "unless-stopped",
                            "unless-stopped" | "no" => "",
                            _ => "",
                        };
                        Some(AppEvent::ImageRunFieldUpdate(ImageRunField::RestartPolicy, next.to_string()))
                    }
                    14 => Some(AppEvent::ImageRunFieldUpdate(ImageRunField::Privileged, String::new())),
                    _ => None,
                }
            })
        }
        (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
            Some(AppEvent::ImageRunToggleAdvanced)
        }
        (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => state.navigation.image_run.as_ref().and_then(|run| {
            let is_text = matches!(run.field_focus, 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 9 | 10 | 11 | 12 | 13);
            if !is_text { return None; }
            let (val, field) = match run.field_focus {
                0 => (run.command.as_str(), ImageRunField::Command),
                1 => (run.shell.as_str(), ImageRunField::Shell),
                2 => (run.user.as_str(), ImageRunField::User),
                3 => (run.workdir.as_str(), ImageRunField::Workdir),
                4 => (run.env_vars.as_str(), ImageRunField::EnvVars),
                5 => (run.port_mapping.as_str(), ImageRunField::PortMapping),
                6 => (run.volumes.as_str(), ImageRunField::Volumes),
                7 => (run.container_name.as_str(), ImageRunField::ContainerName),
                9 => (run.restart_policy.as_str(), ImageRunField::RestartPolicy),
                10 => (run.memory_limit.as_str(), ImageRunField::MemoryLimit),
                11 => (run.cpu_limit.as_str(), ImageRunField::CpuLimit),
                12 => (run.network.as_str(), ImageRunField::Network),
                13 => (run.labels.as_str(), ImageRunField::Labels),
                _ => return None,
            };
            Some(AppEvent::ImageRunFieldUpdate(field, format!("{}{}", val, c)))
        }),
        _ => None,
    }
}
