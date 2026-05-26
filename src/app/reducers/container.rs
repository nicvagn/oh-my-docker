use crate::app::event::{AppEvent, Command};
use crate::app::state::AppState;
use crate::search::fuzzy::Fuzzy;

fn apply_filter(state: &mut AppState) {
    let items = &state.containers.items;
    let filter = &state.containers.filter;
    if filter.is_empty() {
        state.containers.filtered = (0..items.len()).collect();
    } else {
        let fuzzy = Fuzzy::new();
        let results = fuzzy.filter(filter, items, |c| &c.name);
        if results.is_empty() {
            let results = fuzzy.filter(filter, items, |c| &c.image);
            state.containers.filtered = results.into_iter().map(|(i, _)| i).collect();
        } else {
            state.containers.filtered = results.into_iter().map(|(i, _)| i).collect();
        }
    }
    if state.containers.selected >= state.containers.filtered.len() {
        state.containers.selected = state.containers.filtered.len().saturating_sub(1);
    }
}

pub fn reduce(state: &mut AppState, event: &AppEvent) -> Vec<Command> {
    let mut commands = Vec::new();
    match event {
        AppEvent::ContainersUpdated(containers) => {
            state.containers.items = containers.clone();
            state.containers.loading = false;
            state.containers.docker_connected = true;
            apply_filter(state);
        }
        AppEvent::SelectContainer(idx) if *idx < state.containers.filtered.len() => {
            state.containers.selected = *idx;
        }
        AppEvent::FilterContainers(q) => {
            state.containers.filter = q.clone();
            if state.containers.filter.is_empty() {
                state.containers.filter_active = false;
            }
            apply_filter(state);
        }
        AppEvent::ActivateFilter => {
            state.containers.filter_active = true;
        }
        AppEvent::RestartContainer(id) => commands.push(Command::RestartContainer(id.clone())),
        AppEvent::StopContainer(id) => {
            state.containers.stopping_containers.insert(id.clone());
            commands.push(Command::StopContainer(id.clone()));
        }
        AppEvent::ContainerStopped(id) => {
            state.containers.stopping_containers.remove(id);
            state.containers.starting_containers.remove(id);
        }
        AppEvent::ContainerStarted(id) => {
            state.containers.starting_containers.remove(id);
        }
        AppEvent::StartContainer(id) => commands.push(Command::StartContainer(id.clone())),
        AppEvent::ContainerDeleted(id) => {
            state.containers.deleting_containers.remove(id);
        }
        AppEvent::ToggleSelectionMode => {
            state.containers.selection_mode = !state.containers.selection_mode;
            if !state.containers.selection_mode {
                state.containers.selected_ids.clear();
            }
        }
        AppEvent::ToggleSelectContainer(id) if state.containers.selection_mode => {
            if state.containers.selected_ids.contains(id) {
                state.containers.selected_ids.remove(id);
            } else {
                state.containers.selected_ids.insert(id.clone());
            }
        }
        AppEvent::SelectAllContainers if state.containers.selection_mode => {
            for &idx in &state.containers.filtered {
                if let Some(c) = state.containers.items.get(idx) {
                    state.containers.selected_ids.insert(c.id.clone());
                }
            }
        }
        AppEvent::BatchToggleContainers(ids) => {
            for id in ids {
                let is_running = state.containers.items.iter()
                    .find(|c| c.id == *id)
                    .map(|c| c.state == "running")
                    .unwrap_or(false);
                if is_running {
                    state.containers.stopping_containers.insert(id.clone());
                } else {
                    state.containers.starting_containers.insert(id.clone());
                }
            }
            commands.push(Command::BatchToggleContainers(ids.clone()));
        }
        _ => {}
    }
    commands
}
