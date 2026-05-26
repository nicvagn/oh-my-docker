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
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                let next = (state.images.selected + 1).min(state.images.filtered.len().saturating_sub(1));
                Some(AppEvent::SelectImage(next))
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let prev = state.images.selected.saturating_sub(1);
                Some(AppEvent::SelectImage(prev))
            }
            KeyCode::Char('r') => {
                state.images.filtered.get(state.images.selected)
                    .and_then(|&idx| state.images.items.get(idx))
                    .map(|img| AppEvent::RunImage(img.repository.clone(), img.tag.clone()))
            }
            KeyCode::Char('d') => {
                state.images.filtered.get(state.images.selected)
                    .and_then(|&idx| state.images.items.get(idx))
                    .map(|img| AppEvent::ShowConfirmDialog(
                        format!("Remove image '{}:{}'?", img.repository, img.tag),
                        ConfirmAction::RemoveImage(img.id.clone()),
                    ))
            }
            KeyCode::Char('D') => Some(AppEvent::ShowConfirmDialog(
                "Remove all dangling (<none>) images?".to_string(),
                ConfirmAction::RemoveDanglingImages,
            )),
            KeyCode::Char('p') => Some(AppEvent::ShowConfirmDialog(
                "Prune all unused images?".to_string(),
                ConfirmAction::PruneUnusedImages,
            )),
            KeyCode::Char('/') => Some(AppEvent::ActivateImageFilter),
            KeyCode::Enter => {
                state.images.filtered.get(state.images.selected)
                    .and_then(|&idx| state.images.items.get(idx))
                    .map(|img| AppEvent::RunImage(img.repository.clone(), img.tag.clone()))
            }
            _ => None,
        }
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
            if c == 'a' && run.field_focus == 8 {
                return AppEvent::ImageRunToggleAutoremove;
            }
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
            AppEvent::ImageRunFieldUpdate(field, format!("{}{}", val, c))
        }),
        _ => None,
    }
}
