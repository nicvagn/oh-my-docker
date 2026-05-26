use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::event::{AppEvent, ConfirmAction, ImageRunField};
use crate::app::state::AppState;

pub fn handle_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    if state.images.filter_active {
        match (key.code, key.modifiers) {
            (KeyCode::Backspace, _) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                let new_q = state.images.filter.chars().take(state.images.filter.chars().count().saturating_sub(1)).collect();
                Some(AppEvent::FilterImages(new_q))
            }
            (KeyCode::Esc, _) => Some(AppEvent::FilterImages(String::new())),
            (KeyCode::Enter, _) => Some(AppEvent::FilterSubmit(None)),
            (KeyCode::Down, _) => {
                let next = (state.images.selected + 1).min(state.images.filtered.len().saturating_sub(1));
                Some(AppEvent::FilterSubmit(Some(next)))
            }
            (KeyCode::Up, _) => {
                let prev = state.images.selected.saturating_sub(1);
                Some(AppEvent::FilterSubmit(Some(prev)))
            }
            (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                let new_q = format!("{}{}", state.images.filter, c);
                Some(AppEvent::FilterImages(new_q))
            }
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
        if km.is_run_image(code, mods) {
            return state.images.filtered.get(state.images.selected)
                .and_then(|&idx| state.images.items.get(idx))
                .map(|img| AppEvent::RunImage(img.repository.clone(), img.tag.clone()));
        }
        if km.is_remove_image(code, mods) {
            return state.images.filtered.get(state.images.selected)
                .and_then(|&idx| state.images.items.get(idx))
                .map(|img| AppEvent::ShowConfirmDialog(
                    format!("Remove image '{}:{}'?", img.repository, img.tag),
                    ConfirmAction::RemoveImage(img.id.clone()),
                ));
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
        if km.is_search(code, mods) {
            return Some(AppEvent::ActivateImageFilter);
        }
        if code == KeyCode::Enter {
            return state.images.filtered.get(state.images.selected)
                .and_then(|&idx| state.images.items.get(idx))
                .map(|img| AppEvent::RunImage(img.repository.clone(), img.tag.clone()));
        }
        None
    }
}

pub fn handle_image_run_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    match (key.code, key.modifiers) {
        (KeyCode::Backspace, _) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => state.navigation.image_run.as_ref().map(|run| {
            let (val, field) = match run.field_focus {
                0 => (run.command.as_str(), ImageRunField::Command),
                1 => (run.shell.as_str(), ImageRunField::Shell),
                2 => (run.user.as_str(), ImageRunField::User),
                3 => (run.workdir.as_str(), ImageRunField::Workdir),
                4 => (run.env_vars.as_str(), ImageRunField::EnvVars),
                5 => (run.port_mapping.as_str(), ImageRunField::PortMapping),
                6 => (run.volumes.as_str(), ImageRunField::Volumes),
                _ => (run.container_name.as_str(), ImageRunField::ContainerName),
            };
            let new_val: String = val.chars().take(val.chars().count().saturating_sub(1)).collect();
            AppEvent::ImageRunFieldUpdate(field, new_val)
        }),
        (KeyCode::Esc, _) => Some(AppEvent::Back),
        (KeyCode::Enter, _) => Some(AppEvent::ImageRunSubmit),
        (KeyCode::Tab, _) | (KeyCode::Down, _) => Some(AppEvent::ImageRunFocusNext),
        (KeyCode::Up, _) => Some(AppEvent::ImageRunFocusPrev),
        (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => state.navigation.image_run.as_ref().map(|run| {
            if c == 'a' {
                if run.field_focus == 8 {
                    return AppEvent::ImageRunToggleAutoremove;
                }
                if run.field_focus == 14 {
                    return AppEvent::ImageRunFieldUpdate(ImageRunField::Privileged, String::new());
                }
            }
            if c == 'a' && run.field_focus == 8 && !run.show_advanced {
                return AppEvent::ImageRunToggleAdvanced;
            }
            let (val, field) = match run.field_focus {
                0 => (run.command.as_str(), ImageRunField::Command),
                1 => (run.shell.as_str(), ImageRunField::Shell),
                2 => (run.user.as_str(), ImageRunField::User),
                3 => (run.workdir.as_str(), ImageRunField::Workdir),
                4 => (run.env_vars.as_str(), ImageRunField::EnvVars),
                5 => (run.port_mapping.as_str(), ImageRunField::PortMapping),
                6 => (run.volumes.as_str(), ImageRunField::Volumes),
                7 => (run.container_name.as_str(), ImageRunField::ContainerName),
                8 => (run.restart_policy.as_str(), ImageRunField::RestartPolicy),
                9 => (run.memory_limit.as_str(), ImageRunField::MemoryLimit),
                10 => (run.cpu_limit.as_str(), ImageRunField::CpuLimit),
                11 => (run.network.as_str(), ImageRunField::Network),
                12 => (run.labels.as_str(), ImageRunField::Labels),
                _ => (run.container_name.as_str(), ImageRunField::ContainerName),
            };
            AppEvent::ImageRunFieldUpdate(field, format!("{}{}", val, c))
        }),
        _ => None,
    }
}
