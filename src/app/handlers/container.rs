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
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                let next = (state.containers.selected + 1).min(state.containers.filtered.len().saturating_sub(1));
                Some(AppEvent::SelectContainer(next))
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let prev = state.containers.selected.saturating_sub(1);
                Some(AppEvent::SelectContainer(prev))
            }
            KeyCode::Enter => Some(AppEvent::ShowDetails),
            KeyCode::Char('/') => Some(AppEvent::ActivateFilter),
            KeyCode::Char('l') => {
                state.containers.filtered.get(state.containers.selected)
                    .and_then(|&idx| state.containers.items.get(idx))
                    .map(|c| AppEvent::Navigate(Mode::Logs(c.id.clone())))
            }
            KeyCode::Char('s') => {
                state.containers.filtered.get(state.containers.selected)
                    .and_then(|&idx| state.containers.items.get(idx))
                    .map(|c| AppEvent::Navigate(Mode::ShellConfig(c.id.clone())))
            }
            KeyCode::Char('r') => {
                state.containers.filtered.get(state.containers.selected)
                    .and_then(|&idx| state.containers.items.get(idx))
                    .map(|c| AppEvent::RestartContainer(c.id.clone()))
            }
            KeyCode::Char('t') => {
                if state.containers.selection_mode {
                    let ids: Vec<String> = state.containers.selected_ids.iter().cloned().collect();
                    if ids.is_empty() {
                        state.containers.filtered.get(state.containers.selected)
                            .and_then(|&idx| state.containers.items.get(idx))
                            .map(|c| if c.state == "running" {
                                AppEvent::StopContainer(c.id.clone())
                            } else {
                                AppEvent::StartContainer(c.id.clone())
                            })
                    } else {
                        Some(AppEvent::BatchToggleContainers(ids))
                    }
                } else {
                    state.containers.filtered.get(state.containers.selected)
                        .and_then(|&idx| state.containers.items.get(idx))
                        .map(|c| if c.state == "running" {
                            AppEvent::StopContainer(c.id.clone())
                        } else {
                            AppEvent::StartContainer(c.id.clone())
                        })
                }
            }
            KeyCode::Char('d') => {
                if state.containers.selection_mode {
                    let ids: Vec<String> = state.containers.selected_ids.iter().cloned().collect();
                    if ids.is_empty() {
                        state.containers.filtered.get(state.containers.selected)
                            .and_then(|&idx| state.containers.items.get(idx))
                            .map(|c| AppEvent::ShowConfirmDialog(
                                format!("Delete container '{}'?", c.name),
                                ConfirmAction::DeleteContainer(c.id.clone()),
                            ))
                    } else {
                        Some(AppEvent::ShowConfirmDialog(
                            format!("Delete {} selected container(s)?", ids.len()),
                            ConfirmAction::BatchDeleteContainers,
                        ))
                    }
                } else {
                    state.containers.filtered.get(state.containers.selected)
                        .and_then(|&idx| state.containers.items.get(idx))
                        .map(|c| AppEvent::ShowConfirmDialog(
                            format!("Delete container '{}'?", c.name),
                            ConfirmAction::DeleteContainer(c.id.clone()),
                        ))
                }
            }
            KeyCode::Char(' ') => {
                if !state.containers.selection_mode {
                    Some(AppEvent::ToggleSelectionMode)
                } else {
                    state.containers.filtered.get(state.containers.selected)
                        .and_then(|&idx| state.containers.items.get(idx))
                        .map(|c| AppEvent::ToggleSelectContainer(c.id.clone()))
                }
            }
            KeyCode::Char('a') if key.modifiers == KeyModifiers::CONTROL => {
                if state.containers.selection_mode {
                    Some(AppEvent::SelectAllContainers)
                } else {
                    None
                }
            }
            KeyCode::Char('i') => Some(AppEvent::Navigate(Mode::Images)),
            KeyCode::Char('e') => Some(AppEvent::Navigate(Mode::Events)),
            KeyCode::Char('%') => Some(AppEvent::Navigate(Mode::Statistics)),
            KeyCode::Char('n') => Some(AppEvent::Navigate(Mode::Networks)),
            KeyCode::Char('v') => Some(AppEvent::Navigate(Mode::Volumes)),
            KeyCode::Esc => {
                if state.containers.selection_mode {
                    Some(AppEvent::ToggleSelectionMode)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
