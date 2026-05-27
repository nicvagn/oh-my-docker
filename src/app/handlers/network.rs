use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::event::{AppEvent, ConfirmAction};
use crate::app::state::AppState;

pub fn handle_key_with_clipboard(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('y') {
        if let Some(n) = state.networks.items.get(state.networks.selected) {
            let _ = crate::util::copy_to_clipboard(&n.id);
            return Some(AppEvent::Info(format!("Network ID copied to clipboard")));
        }
    }
    handle_key(key, state)
}

pub fn handle_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    let km = &state.keymap;
    let code = key.code;
    let mods = key.modifiers;

    if code == KeyCode::Esc {
        return Some(AppEvent::Back);
    }
    if km.is_navigate_down(code, mods) || code == KeyCode::Down {
        let next = (state.networks.selected + 1).min(state.networks.items.len().saturating_sub(1));
        return Some(AppEvent::SelectNetwork(next));
    }
    if km.is_navigate_up(code, mods) || code == KeyCode::Up {
        let prev = state.networks.selected.saturating_sub(1);
        return Some(AppEvent::SelectNetwork(prev));
    }
    if km.is_delete(code, mods) {
        return state.networks.items.get(state.networks.selected)
            .map(|n| AppEvent::ShowConfirmDialog(
                format!("Remove network '{}'?", n.name),
                ConfirmAction::RemoveNetwork(n.id.clone()),
            ));
    }
    None
}
