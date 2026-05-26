use crossterm::event::{KeyCode, KeyEvent};
use crate::app::event::{AppEvent, ConfirmAction};
use crate::app::state::AppState;

pub fn handle_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    match key.code {
        KeyCode::Esc => Some(AppEvent::Back),
        KeyCode::Char('j') | KeyCode::Down => {
            let next = (state.networks.selected + 1).min(state.networks.items.len().saturating_sub(1));
            Some(AppEvent::SelectNetwork(next))
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let prev = state.networks.selected.saturating_sub(1);
            Some(AppEvent::SelectNetwork(prev))
        }
        KeyCode::Char('d') => {
            state.networks.items.get(state.networks.selected)
                .map(|n| AppEvent::ShowConfirmDialog(
                    format!("Remove network '{}'?", n.name),
                    ConfirmAction::RemoveNetwork(n.id.clone()),
                ))
        }
        _ => None,
    }
}
