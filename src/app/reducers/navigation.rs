use std::collections::VecDeque;

use crate::app::event::{AppEvent, Command, ConfirmAction};
use crate::app::mode;
use crate::app::mode::Mode;
use crate::app::state::AppState;

pub fn reduce(state: &mut AppState, event: &AppEvent) -> Vec<Command> {
    let mut commands = Vec::new();
    match event {
        AppEvent::Navigate(mode @ Mode::Shell(_)) => {
            if let Mode::Shell(id) = &mode {
                state.navigation.shell = Some(crate::app::state::ShellState {
                    container_id: id.clone(),
                    active: true,
                    stop_on_exit: false,
                    shell: "bash".to_string(),
                    user: String::new(),
                    workdir: String::new(),
                });
            }
            state.navigation.mode_stack.push(mode.clone());
        }
        AppEvent::Navigate(mode @ Mode::Logs(_)) => {
            if let Mode::Logs(id) = &mode {
                if let Some(ref logs) = state.navigation.logs {
                    if let Some(handle) = state.log_streams.remove(&logs.container_id) {
                        handle.abort();
                    }
                }
                state.navigation.logs = Some(crate::app::state::LogState {
                    container_id: id.clone(),
                    buffer: VecDeque::new(),
                    max_lines: 10000,
                    paused: false,
                    search: String::new(),
                    search_active: false,
                    scroll_offset: 0,
                    tail: true,
                    show_timestamps: false,
                    viewport_height: 0,
                });
                commands.push(Command::FetchLogs(id.clone()));
            }
            state.navigation.mode_stack.push(mode.clone());
        }
        AppEvent::Navigate(mode @ Mode::ShellConfig(_)) => {
            if let Mode::ShellConfig(id) = &mode {
                let container = state.containers.items.iter().find(|c| c.id == *id);
                let name = container.map(|c| c.name.clone()).unwrap_or_default();
                let image_base = container.map(|c| crate::util::image_base_name(&c.image).to_string());
                let latest = state.config.latest_shell.clone().unwrap_or_else(|| "bash".to_string());
                let per_container = state.config.containers.get(&name).cloned().unwrap_or_default();
                let per_image = image_base.as_ref()
                    .and_then(|ib| state.config.images.get(ib).cloned())
                    .unwrap_or_default();
                let shell = per_container.shell.clone()
                    .or_else(|| per_image.shell.clone())
                    .unwrap_or(latest);
                let user = per_container.user.clone()
                    .or_else(|| per_image.user.clone())
                    .unwrap_or_default();
                let workdir = per_container.workdir.clone()
                    .or_else(|| per_image.workdir.clone())
                    .unwrap_or_default();
                state.navigation.shell_config = Some(crate::app::state::ShellConfigState {
                    container_id: id.clone(),
                    shell,
                    user,
                    workdir,
                    field_focus: 0,
                });
            }
            state.navigation.mode_stack.push(mode.clone());
        }
        AppEvent::Navigate(mode @ Mode::Explorer(_)) => {
            if let Mode::Explorer(id) = &mode {
                state.explorer.container_id = id.clone();
                state.explorer.host.path = std::env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|_| ".".to_string());
                state.explorer.host.items.clear();
                state.explorer.host.selected = 0;
                state.explorer.host.loading = true;
                state.explorer.container.path = "/".to_string();
                state.explorer.container.items.clear();
                state.explorer.container.selected = 0;
                state.explorer.container.loading = true;
                state.explorer.transfer_message = None;
                state.explorer.transfer_error = None;
                state.explorer.transfer_message_clear_tick = 0;
                state.explorer.transfer_error_clear_tick = 0;
                state.explorer.last_click_time = None;
                state.explorer.focus = crate::app::state::ExplorerFocus::Left;
                commands.push(Command::ListHostDir(state.explorer.host.path.clone()));
                commands.push(Command::FetchContainerWorkingDir(id.clone()));
            }
            state.navigation.mode_stack.push(mode.clone());
        }
        AppEvent::Navigate(mode @ Mode::ExplorerVolume(_, _)) => {
            if let Mode::ExplorerVolume(_mountpoint, name) = &mode {
                state.explorer.container_id = format!("__volume__:{}", name);
                state.explorer.host.path = std::env::current_dir()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                state.explorer.host.items.clear();
                state.explorer.host.selected = 0;
                state.explorer.host.loading = true;
                state.explorer.container.path = "/".to_string();
                state.explorer.container.items.clear();
                state.explorer.container.selected = 0;
                state.explorer.container.loading = true;
                state.explorer.transfer_message = None;
                state.explorer.transfer_error = None;
                state.explorer.transfer_message_clear_tick = 0;
                state.explorer.transfer_error_clear_tick = 0;
                state.explorer.last_click_time = None;
                state.explorer.focus = crate::app::state::ExplorerFocus::Left;
                commands.push(Command::ListHostDir(state.explorer.host.path.clone()));
                commands.push(Command::ListVolumeDir(name.clone(), "/".to_string()));
            }
            state.navigation.mode_stack.push(mode.clone());
        }
        AppEvent::Navigate(mode) => {
            if mode::mode_to_tab(mode).is_some() {
                if *mode == Mode::Help {
                    state.previous_tab = state.selected_tab;
                }
                let cur_is_base = mode::mode_to_tab(state.navigation.mode_stack.current()).is_some();
                if cur_is_base {
                    state.navigation.mode_stack.replace_current(mode.clone());
                } else {
                    state.navigation.mode_stack.push(mode.clone());
                }
                state.selected_tab = mode::mode_to_tab(mode).unwrap_or(0);
            } else {
                state.navigation.mode_stack.push(mode.clone());
            }
        }
        AppEvent::Back => {
            use crate::app::mode::Mode;
            if let Mode::ExplorerVolume(_, name) = state.navigation.mode_stack.current() {
                commands.push(Command::RemoveVolumeHelper(name.clone()));
            }
            if let Some(ref logs) = state.navigation.logs {
                if let Some(handle) = state.log_streams.remove(&logs.container_id) {
                    handle.abort();
                }
            }
            state.navigation.logs = None;
            state.navigation.shell_config = None;
            state.navigation.image_run = None;
            state.navigation.diagnostics = None;
            state.navigation.mode_stack.back();
            // Update selected_tab based on the mode we returned to
            if let Some(tab) = mode::mode_to_tab(state.navigation.mode_stack.current()) {
                state.selected_tab = tab;
            }
        }

        AppEvent::ShowHelp => state.navigation.mode_stack.push(Mode::Help),
        AppEvent::HideHelp if *state.navigation.mode_stack.current() == Mode::Help => {
            state.navigation.mode_stack.back();
        }
        AppEvent::ScrollHelp(delta) => {
            state.navigation.help.scroll_offset = crate::util::scroll_offset(state.navigation.help.scroll_offset, *delta, 10000);
        }

        AppEvent::ShowConfirmDialog(prompt, action) => {
            state.navigation.mode_stack.push(Mode::ConfirmDialog { prompt: prompt.clone(), action: action.clone() });
        }
        AppEvent::ConfirmYes => {
            if let Mode::ConfirmDialog { action, .. } = state.navigation.mode_stack.current() {
                let action = action.clone();
                state.navigation.mode_stack.back();
                match action {
                    ConfirmAction::DeleteContainer(id) => {
                        if id.is_empty() {
                            if !state.container_extra.selected_ids.is_empty() {
                                let ids: Vec<String> = state.container_extra.selected_ids.iter().cloned().collect();
                                state.container_extra.selected_ids.clear();
                                state.container_extra.selection_mode = false;
                                commands.push(Command::BatchDeleteContainers(ids));
                            }
                        } else {
                            state.container_extra.deleting_containers.insert(id.clone());
                            commands.push(Command::DeleteContainer(id));
                        }
                    }
                    ConfirmAction::BatchDeleteContainers => {
                        if !state.container_extra.selected_ids.is_empty() {
                            let ids: Vec<String> = state.container_extra.selected_ids.iter().cloned().collect();
                            state.container_extra.selected_ids.clear();
                            state.container_extra.selection_mode = false;
                            commands.push(Command::BatchDeleteContainers(ids));
                        }
                    }
                    ConfirmAction::RemoveImage(id) => {
                        if id.is_empty() {
                            commands.push(Command::RemoveDanglingImages);
                        } else {
                            commands.push(Command::RemoveImage(id));
                        }
                    }
                    ConfirmAction::RemoveDanglingImages => commands.push(Command::RemoveDanglingImages),
                    ConfirmAction::PruneUnusedImages => commands.push(Command::PruneUnusedImages),
                    ConfirmAction::RemoveNetwork(id) => commands.push(Command::RemoveNetwork(id)),
                    ConfirmAction::RemoveVolume(name) => commands.push(Command::RemoveVolume(name)),
                    ConfirmAction::BatchDeleteImages => {
                        let ids: Vec<String> = state.images.selected_ids.iter().cloned().collect();
                        state.images.selected_ids.clear();
                        state.images.selection_mode = false;
                        if !ids.is_empty() {
                            commands.push(Command::BatchDeleteImages(ids));
                        }
                    }
                    ConfirmAction::BatchDeleteNetworks => {
                        let ids: Vec<String> = state.networks.selected_ids.iter().cloned().collect();
                        state.networks.selected_ids.clear();
                        state.networks.selection_mode = false;
                        if !ids.is_empty() {
                            commands.push(Command::BatchDeleteNetworks(ids));
                        }
                    }
                    ConfirmAction::BatchDeleteVolumes => {
                        let ids: Vec<String> = state.volumes.selected_ids.iter().cloned().collect();
                        state.volumes.selected_ids.clear();
                        state.volumes.selection_mode = false;
                        if !ids.is_empty() {
                            commands.push(Command::BatchDeleteVolumes(ids));
                        }
                    }
                    ConfirmAction::DeleteHostFile(path) => commands.push(Command::DeleteHostFile(path)),
                    ConfirmAction::DeleteContainerFile(id, path) => commands.push(Command::DeleteContainerFile(id, path)),
                }
            }
        }
        AppEvent::ConfirmNo => {
            if matches!(state.navigation.mode_stack.current(), Mode::ConfirmDialog { .. }) {
                state.navigation.mode_stack.back();
            }
        }

        AppEvent::ShowDetails => {
            if let Some(&idx) = state.containers.filtered.get(state.containers.selected) {
                if let Some(container) = state.containers.items.get(idx) {
                    let id = container.id.clone();
                    state.navigation.mode_stack.push(Mode::ContainerDetails(id.clone()));
                    state.navigation.details = Some(crate::app::state::DetailsState {
                        id: id.clone(),
                        container_id: container.name.clone(),
                        json: None,
                        scroll_offset: 0,
                    });
                    commands.push(Command::InspectContainer(id));
                }
            }
        }
        AppEvent::Inspected(json, name) => {
            let prev = state.navigation.details.take();
            let existing_id = prev.as_ref().map(|d| d.id.clone()).unwrap_or_default();
            let existing_scroll = prev.as_ref().map(|d| d.scroll_offset).unwrap_or(0);
            state.navigation.details = Some(crate::app::state::DetailsState {
                id: existing_id,
                container_id: name.clone(),
                json: Some(serde_json::to_string_pretty(&json).unwrap_or_default()),
                scroll_offset: existing_scroll,
            });
        }
        AppEvent::ScrollDetails(delta) => {
            if let Some(ref mut d) = state.navigation.details {
                d.scroll_offset = crate::util::scroll_offset(d.scroll_offset, *delta, 10000);
            }
        }

        AppEvent::FilterSubmit(sel) => {
            match state.navigation.mode_stack.current() {
                Mode::Containers => {
                    state.containers.filter_active = false;
                    if let Some(idx) = sel {
                        if *idx < state.containers.filtered.len() {
                            state.containers.selected = *idx;
                        }
                    }
                }
                Mode::Images => {
                    state.images.filter_active = false;
                    if let Some(idx) = sel {
                        if *idx < state.images.filtered.len() {
                            state.images.selected = *idx;
                        }
                    }
                }
                _ => {}
            }
        }

        AppEvent::JumpTop => {
            state.events.scroll_offset = state.events.buffer.len();
            if let Some(ref mut log) = state.navigation.logs {
                log.scroll_offset = log.buffer.len();
                log.tail = false;
            }
        }
        AppEvent::JumpBottom => {
            state.events.scroll_offset = 0;
            if let Some(ref mut log) = state.navigation.logs {
                log.scroll_offset = 0;
                log.tail = true;
            }
        }
        _ => {}
    }
    commands
}
