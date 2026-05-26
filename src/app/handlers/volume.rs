use crossterm::event::{KeyCode, KeyEvent};
use crate::app::event::{AppEvent, ConfirmAction};
use crate::app::state::AppState;

pub fn handle_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    match key.code {
        KeyCode::Esc => Some(AppEvent::Back),
        KeyCode::Char('j') | KeyCode::Down => {
            let next = (state.volumes.selected + 1).min(state.volumes.items.len().saturating_sub(1));
            Some(AppEvent::SelectVolume(next))
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let prev = state.volumes.selected.saturating_sub(1);
            Some(AppEvent::SelectVolume(prev))
        }
        KeyCode::Char('d') => {
            state.volumes.items.get(state.volumes.selected)
                .map(|v| AppEvent::ShowConfirmDialog(
                    format!("Remove volume '{}'?", v.name),
                    ConfirmAction::RemoveVolume(v.name.clone()),
                ))
        }
        _ => None,
    }
}
