use std::path::PathBuf;

use crate::app::event::{AppEvent, Command};
use crate::app::state::{AppState, ExplorerEntry};
use crate::search::fuzzy::Fuzzy;

fn apply_filter(items: &[ExplorerEntry], filter: &str) -> Vec<ExplorerEntry> {
    if filter.is_empty() {
        items.to_vec()
    } else {
        let fuzzy = Fuzzy::new();
        items.iter()
            .filter(|e| fuzzy.score(filter, &e.name).is_some())
            .cloned()
            .collect()
    }
}

fn host_entry_index(state: &AppState) -> Option<usize> {
    let show_parent = state.explorer.host.path != "/";
    let idx = if show_parent { state.explorer.host.selected.saturating_sub(1) } else { state.explorer.host.selected };
    if idx < state.explorer.host.items.len() { Some(idx) } else { None }
}

fn container_entry_index(state: &AppState) -> Option<usize> {
    let show_parent = state.explorer.container.path != "/";
    let idx = if show_parent { state.explorer.container.selected.saturating_sub(1) } else { state.explorer.container.selected };
    if idx < state.explorer.container.items.len() { Some(idx) } else { None }
}

pub fn reduce(state: &mut AppState, event: &AppEvent) -> Vec<Command> {
    let mut commands = Vec::new();

    match event {
        AppEvent::ExplorerSelect => {
            state.explorer.focus = match state.explorer.focus {
                crate::app::state::ExplorerFocus::Left => crate::app::state::ExplorerFocus::Right,
                crate::app::state::ExplorerFocus::Right => crate::app::state::ExplorerFocus::Left,
            };
        }

        AppEvent::ExplorerHostSelect(idx) => {
            state.explorer.host.selected = *idx;
        }

        AppEvent::ExplorerContainerSelect(idx) => {
            state.explorer.container.selected = *idx;
        }

        AppEvent::ExplorerHostGoUp => {
            if let Some(parent) = PathBuf::from(&state.explorer.host.path).parent()
                .and_then(|p| p.to_str())
                .filter(|s| !s.is_empty())
            {
                state.explorer.host.path = if parent == "/" { "/".to_string() } else { parent.to_string() };
                state.explorer.host.selected = 0;
                state.explorer.host.filter = String::new();
                state.explorer.host.filter_active = false;
                state.explorer.host.rename_active = false;
                state.explorer.host.rename_buffer = String::new();
                commands.push(Command::ListHostDir(state.explorer.host.path.clone()));
            }
        }

        AppEvent::ExplorerContainerGoUp => {
            let path = &state.explorer.container.path;
            if path == "/" {
                return commands;
            }
            let cleaned = path.strip_suffix('/').unwrap_or(path);
            let parts: Vec<&str> = cleaned.split('/').filter(|s| !s.is_empty()).collect();
            if parts.len() <= 1 {
                state.explorer.container.path = "/".to_string();
                state.explorer.container.selected = 0;
                state.explorer.container.filter = String::new();
                state.explorer.container.filter_active = false;
                state.explorer.container.rename_active = false;
                state.explorer.container.rename_buffer = String::new();
                commands.push(Command::ListContainerDir(
                    state.explorer.container_id.clone(),
                    "/".to_string(),
                ));
                return commands;
            }
            let parent = format!("/{}/", parts[..parts.len() - 1].join("/"));
            let parent_clone = parent.clone();
            state.explorer.container.path = parent;
            state.explorer.container.selected = 0;
            state.explorer.container.filter = String::new();
            state.explorer.container.filter_active = false;
            state.explorer.container.rename_active = false;
            state.explorer.container.rename_buffer = String::new();
            commands.push(Command::ListContainerDir(
                state.explorer.container_id.clone(),
                parent_clone,
            ));
        }

        AppEvent::ExplorerHostEnterDir(name) => {
            let new_path = if state.explorer.host.path == "/" {
                format!("/{}", name)
            } else {
                format!("{}/{}", state.explorer.host.path, name)
            };
            let new_path_clone = new_path.clone();
            state.explorer.host.path = new_path;
            state.explorer.host.selected = 0;
            state.explorer.host.filter = String::new();
            state.explorer.host.filter_active = false;
            state.explorer.host.rename_active = false;
            state.explorer.host.rename_buffer = String::new();
            commands.push(Command::ListHostDir(new_path_clone));
        }

        AppEvent::ExplorerContainerEnterDir(name) => {
            let path = &state.explorer.container.path;
            let new_path = if path.ends_with('/') {
                format!("{}{}", path, name)
            } else {
                format!("{}/{}", path, name)
            };
            state.explorer.container.path = format!("{}/", new_path);
            state.explorer.container.selected = 0;
            state.explorer.container.filter = String::new();
            state.explorer.container.filter_active = false;
            state.explorer.container.rename_active = false;
            state.explorer.container.rename_buffer = String::new();
            commands.push(Command::ListContainerDir(
                state.explorer.container_id.clone(),
                state.explorer.container.path.clone(),
            ));
        }

        AppEvent::ExplorerHostRefresh => {
            commands.push(Command::ListHostDir(state.explorer.host.path.clone()));
        }

        AppEvent::ExplorerContainerRefresh => {
            commands.push(Command::ListContainerDir(
                state.explorer.container_id.clone(),
                state.explorer.container.path.clone(),
            ));
        }

        AppEvent::ExplorerHostActivateFilter => {
            state.explorer.host.filter_active = true;
        }

        AppEvent::ExplorerContainerActivateFilter => {
            state.explorer.container.filter_active = true;
        }

        AppEvent::ExplorerFilter(q) => {
            if state.explorer.host.filter_active {
                state.explorer.host.filter = q.clone();
                if !q.is_empty() {
                    state.explorer.host.items = apply_filter(&state.explorer.host.all_items, q);
                }
            } else if state.explorer.container.filter_active {
                state.explorer.container.filter = q.clone();
                if !q.is_empty() {
                    state.explorer.container.items = apply_filter(&state.explorer.container.all_items, q);
                }
            }
            if q.is_empty() {
                state.explorer.host.filter_active = false;
                state.explorer.container.filter_active = false;
                state.explorer.host.items = state.explorer.host.all_items.clone();
                state.explorer.container.items = state.explorer.container.all_items.clone();
            }
        }

        AppEvent::ExplorerFilterSubmit => {
            state.explorer.host.filter_active = false;
            state.explorer.container.filter_active = false;
        }

        AppEvent::ExplorerHostActivateRename => {
            let panel = &mut state.explorer.host;
            let show_parent = panel.path != "/";
            let entry_idx = if show_parent { panel.selected.saturating_sub(1) } else { panel.selected };
            if let Some(entry) = panel.items.get(entry_idx) {
                panel.rename_buffer = entry.name.clone();
                panel.rename_active = true;
            }
        }

        AppEvent::ExplorerContainerActivateRename => {
            let panel = &mut state.explorer.container;
            let show_parent = panel.path != "/";
            let entry_idx = if show_parent { panel.selected.saturating_sub(1) } else { panel.selected };
            if let Some(entry) = panel.items.get(entry_idx) {
                panel.rename_buffer = entry.name.clone();
                panel.rename_active = true;
            }
        }

        AppEvent::ExplorerRenameUpdate(q) => {
            if state.explorer.host.rename_active {
                state.explorer.host.rename_buffer = q.clone();
            } else if state.explorer.container.rename_active {
                state.explorer.container.rename_buffer = q.clone();
            }
        }

        AppEvent::ExplorerRenameCancel => {
            if state.explorer.host.rename_active {
                state.explorer.host.rename_active = false;
                state.explorer.host.rename_buffer = String::new();
            }
            if state.explorer.container.rename_active {
                state.explorer.container.rename_active = false;
                state.explorer.container.rename_buffer = String::new();
            }
        }

        AppEvent::ExplorerRenameSubmit => {
            if state.explorer.host.rename_active {
                let panel = &mut state.explorer.host;
                let show_parent = panel.path != "/";
                let entry_idx = if show_parent { panel.selected.saturating_sub(1) } else { panel.selected };
                if let Some(entry) = panel.items.get(entry_idx) {
                    if !panel.rename_buffer.is_empty() && panel.rename_buffer != entry.name {
                        let old_path = if panel.path == "/" {
                            format!("/{}", entry.name)
                        } else {
                            format!("{}/{}", panel.path, entry.name)
                        };
                        let new_path = if panel.path == "/" {
                            format!("/{}", panel.rename_buffer)
                        } else {
                            format!("{}/{}", panel.path, panel.rename_buffer)
                        };
                        commands.push(Command::RenameHostFile(old_path, new_path));
                    }
                }
                panel.rename_active = false;
                panel.rename_buffer = String::new();
            } else if state.explorer.container.rename_active {
                let panel = &mut state.explorer.container;
                let show_parent = panel.path != "/";
                let entry_idx = if show_parent { panel.selected.saturating_sub(1) } else { panel.selected };
                if let Some(entry) = panel.items.get(entry_idx) {
                    if !panel.rename_buffer.is_empty() && panel.rename_buffer != entry.name {
                        let old_path = if panel.path == "/" {
                            format!("/{}", entry.name)
                        } else {
                            format!("{}/{}", panel.path, entry.name)
                        };
                        let new_path = if panel.path == "/" {
                            format!("/{}", panel.rename_buffer)
                        } else {
                            format!("{}/{}", panel.path, panel.rename_buffer)
                        };
                        commands.push(Command::RenameContainerFile(
                            state.explorer.container_id.clone(),
                            old_path,
                            new_path,
                        ));
                    }
                }
                panel.rename_active = false;
                panel.rename_buffer = String::new();
            }
        }
        AppEvent::ExplorerContainerDirUpdated(container_id, path, entries) => {
            if state.explorer.container_id == *container_id
                && state.explorer.container.path == *path
            {
                state.explorer.container.all_items = entries.clone();
                let filter = &state.explorer.container.filter;
                state.explorer.container.items = apply_filter(entries, filter);
                state.explorer.container.selected = 0;
                state.explorer.container.loading = false;
            }
        }

        AppEvent::ExplorerHostDirUpdated(path, entries) => {
            if state.explorer.host.path == *path {
                state.explorer.host.all_items = entries.clone();
                let filter = &state.explorer.host.filter;
                state.explorer.host.items = apply_filter(entries, filter);
                state.explorer.host.selected = 0;
                state.explorer.host.loading = false;
            }
        }

        AppEvent::ExplorerCopyToContainer => {
            if let Some(entry_idx) = host_entry_index(state) {
                if let Some(entry) = state.explorer.host.items.get(entry_idx) {
                    let host_path = if state.explorer.host.path == "/" {
                        format!("/{}", entry.name)
                    } else {
                        format!("{}/{}", state.explorer.host.path, entry.name)
                    };
                    let container_dest = if state.explorer.container.path.ends_with('/') {
                        state.explorer.container.path.clone()
                    } else {
                        format!("{}/", state.explorer.container.path)
                    };
                    commands.push(Command::CopyToContainer(
                        state.explorer.container_id.clone(),
                        host_path,
                        container_dest,
                    ));
                }
            }
        }

        AppEvent::ExplorerCopyFromContainer => {
            if let Some(entry_idx) = container_entry_index(state) {
                if let Some(entry) = state.explorer.container.items.get(entry_idx) {
                    let container_src = if state.explorer.container.path.ends_with('/') {
                        format!("{}{}", state.explorer.container.path, entry.name)
                    } else {
                        format!("{}/{}", state.explorer.container.path, entry.name)
                    };
                    let host_dest = state.explorer.host.path.clone();
                    commands.push(Command::CopyFromContainer(
                        state.explorer.container_id.clone(),
                        container_src,
                        host_dest,
                    ));
                }
            }
        }

        AppEvent::ExplorerTransferComplete(msg) => {
            state.explorer.transfer_message = Some(msg.clone());
            state.explorer.transfer_error = None;
            state.explorer.transfer_message_clear_tick = state.tick_count + 2;
            commands.push(Command::ListHostDir(state.explorer.host.path.clone()));
            commands.push(Command::ListContainerDir(
                state.explorer.container_id.clone(),
                state.explorer.container.path.clone(),
            ));
        }

        AppEvent::ExplorerTransferError(msg) => {
            state.explorer.transfer_error = Some(msg.clone());
            state.explorer.transfer_message = None;
            state.explorer.transfer_error_clear_tick = state.tick_count + 2;
        }

        _ => {}
    }

    commands
}
