use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::event::{AppEvent, ConfirmAction};
use crate::app::state::AppState;
use crate::app::state::ExplorerFocus;

fn max_selected(item_count: usize, show_parent: bool) -> usize {
    if show_parent { item_count } else { item_count.saturating_sub(1) }
}

pub fn handle_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    if state.explorer.host.filter_active || state.explorer.container.filter_active {
        return handle_filter_input(key, state);
    }
    if state.explorer.host.rename_active || state.explorer.container.rename_active {
        return handle_rename_input(key, state);
    }

    let code = key.code;
    let mods = key.modifiers;

    if (code == KeyCode::Tab && mods == KeyModifiers::NONE)
        || code == KeyCode::BackTab
        || (code == KeyCode::Tab && mods == KeyModifiers::SHIFT)
    {
        return Some(AppEvent::ExplorerSelect);
    }

    // Backspace to go up
    if code == KeyCode::Backspace {
        return Some(AppEvent::ExplorerHostGoUp);
    }

    let host_selected = state.explorer.focus == ExplorerFocus::Left;
    let selected = if host_selected { state.explorer.host.selected } else { state.explorer.container.selected };
    let path = if host_selected { &state.explorer.host.path } else { &state.explorer.container.path };
    let items = if host_selected { &state.explorer.host.items } else { &state.explorer.container.items };
    let show_parent = *path != "/";
    let max = if host_selected {
        max_selected(items.len(), show_parent)
    } else {
        max_selected(items.len(), show_parent)
    };

    if key_matches_nav_down(key, &state.keymap) {
        let next = (selected + 1).min(max);
        return Some(if host_selected { AppEvent::ExplorerHostSelect(next) } else { AppEvent::ExplorerContainerSelect(next) });
    }
    if key_matches_nav_up(key, &state.keymap) {
        let prev = selected.saturating_sub(1);
        return Some(if host_selected { AppEvent::ExplorerHostSelect(prev) } else { AppEvent::ExplorerContainerSelect(prev) });
    }

    if code == KeyCode::PageDown {
        let next = (selected + 20).min(max);
        return Some(if host_selected { AppEvent::ExplorerHostSelect(next) } else { AppEvent::ExplorerContainerSelect(next) });
    }
    if code == KeyCode::PageUp {
        let prev = selected.saturating_sub(20);
        return Some(if host_selected { AppEvent::ExplorerHostSelect(prev) } else { AppEvent::ExplorerContainerSelect(prev) });
    }

    // g - jump to top
    if code == KeyCode::Char('g') && mods == KeyModifiers::NONE {
        return Some(if host_selected { AppEvent::ExplorerHostSelect(0) } else { AppEvent::ExplorerContainerSelect(0) });
    }
    // G - jump to bottom
    if code == KeyCode::Char('G') {
        return Some(if host_selected { AppEvent::ExplorerHostSelect(max) } else { AppEvent::ExplorerContainerSelect(max) });
    }

    // Enter - enter directory or go up
    if code == KeyCode::Enter {
        if show_parent && selected == 0 {
            return Some(if host_selected { AppEvent::ExplorerHostGoUp } else { AppEvent::ExplorerContainerGoUp });
        }
        let entry_idx = if show_parent { selected.saturating_sub(1) } else { selected };
        if let Some(entry) = items.get(entry_idx) {
            if entry.is_dir {
                return Some(if host_selected { AppEvent::ExplorerHostEnterDir(entry.name.clone()) } else { AppEvent::ExplorerContainerEnterDir(entry.name.clone()) });
            }
        }
    }
    // r - rename
    if code == KeyCode::Char('r') && mods == KeyModifiers::NONE {
        if show_parent && selected == 0 {
            return None;
        }
        let entry_idx = if show_parent { selected.saturating_sub(1) } else { selected };
        if items.get(entry_idx).is_some() {
            return Some(if host_selected { AppEvent::ExplorerHostActivateRename } else { AppEvent::ExplorerContainerActivateRename });
        }
    }
    // R - refresh
    if code == KeyCode::Char('R') {
        return Some(if host_selected { AppEvent::ExplorerHostRefresh } else { AppEvent::ExplorerContainerRefresh });
    }
    // / - filter
    if code == KeyCode::Char('/') && mods == KeyModifiers::NONE {
        return Some(if host_selected { AppEvent::ExplorerHostActivateFilter } else { AppEvent::ExplorerContainerActivateFilter });
    }
    if code == KeyCode::Char('c') && mods == KeyModifiers::CONTROL {
        return Some(match state.explorer.focus {
            ExplorerFocus::Left => AppEvent::ExplorerCopyToContainer,
            ExplorerFocus::Right => AppEvent::ExplorerCopyFromContainer,
        });
    }
    // d - delete file/directory
    if code == KeyCode::Char('d') && mods == KeyModifiers::NONE {
        if show_parent && selected == 0 {
            return None;
        }
        let entry_idx = if show_parent { selected.saturating_sub(1) } else { selected };
        if let Some(entry) = items.get(entry_idx) {
            let full_path = if *path == "/" || path.is_empty() {
                format!("/{}", entry.name)
            } else {
                format!("{}/{}", path, entry.name)
            };
            if host_selected {
                return Some(AppEvent::ShowConfirmDialog(
                    format!("Delete '{}'?", full_path),
                    ConfirmAction::DeleteHostFile(full_path),
                ));
            } else {
                return Some(AppEvent::ShowConfirmDialog(
                    format!("Delete '{}'?", full_path),
                    ConfirmAction::DeleteContainerFile(
                        state.explorer.container_id.clone(),
                        full_path,
                    ),
                ));
            }
        }
    }

    // Esc - go back
    if code == KeyCode::Esc {
        return Some(AppEvent::Back);
    }

    None
}

fn handle_filter_input(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    let host_active = state.explorer.host.filter_active;
    let current_filter = if host_active {
        &state.explorer.host.filter
    } else {
        &state.explorer.container.filter
    };

    match (key.code, key.modifiers) {
        (KeyCode::Backspace, _) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
            let new_q = current_filter.chars().take(current_filter.chars().count().saturating_sub(1)).collect();
            Some(AppEvent::ExplorerFilter(new_q))
        }
        (KeyCode::Esc, _) => {
            Some(AppEvent::ExplorerFilter(String::new()))
        }
        (KeyCode::Enter, _) => Some(AppEvent::ExplorerFilterSubmit),
        (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
            let new_q = format!("{}{}", current_filter, c);
            Some(AppEvent::ExplorerFilter(new_q))
        }
        _ => None,
    }
}

fn handle_rename_input(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    let host_active = state.explorer.host.rename_active;
    let current = if host_active {
        &state.explorer.host.rename_buffer
    } else {
        &state.explorer.container.rename_buffer
    };

    match (key.code, key.modifiers) {
        (KeyCode::Backspace, _) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
            let new_q = current.chars().take(current.chars().count().saturating_sub(1)).collect();
            Some(AppEvent::ExplorerRenameUpdate(new_q))
        }
        (KeyCode::Esc, _) => {
            Some(AppEvent::ExplorerRenameCancel)
        }
        (KeyCode::Enter, _) => Some(AppEvent::ExplorerRenameSubmit),
        (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
            let new_q = format!("{}{}", current, c);
            Some(AppEvent::ExplorerRenameUpdate(new_q))
        }
        _ => None,
    }
}

fn key_matches_nav_down(key: KeyEvent, km: &crate::input::keymap::KeyMap) -> bool {
    if km.is_navigate_down(key.code, key.modifiers) {
        return true;
    }
    if key.code == KeyCode::Down {
        return true;
    }
    false
}

fn key_matches_nav_up(key: KeyEvent, km: &crate::input::keymap::KeyMap) -> bool {
    if km.is_navigate_up(key.code, key.modifiers) {
        return true;
    }
    if key.code == KeyCode::Up {
        return true;
    }
    false
}
