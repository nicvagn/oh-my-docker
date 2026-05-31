use crate::app::event::{AppEvent, Command};
use crate::app::state::AppState;

fn apply_filter(state: &mut AppState) {
    let _items = &state.containers.items;
    let _filter = &state.containers.filter;

    let status_filter = state.container_extra.status_filter.clone();

    state.containers.apply_filter(|item| {
        if status_filter.is_empty() {
            true
        } else {
            item.state == status_filter
        }
    });

    state.containers.apply_sort();
    state.containers.reorder_by_group();
}

pub fn reduce(state: &mut AppState, event: &AppEvent) -> Vec<Command> {
    let mut commands = Vec::new();
    match event {
        AppEvent::ContainersRefreshNeeded => {
            commands.push(Command::RefreshContainers);
        }
        AppEvent::ContainersUpdated(containers) => {
            let prev_selected_id = state.containers.filtered.get(state.containers.selected)
                .and_then(|&idx| state.containers.items.get(idx))
                .map(|c| c.id.clone());

            state.containers.items = containers.clone();
            state.containers.loading = false;
            state.containers.last_updated = Some(std::time::Instant::now());
            state.container_extra.docker_connected = true;
            apply_filter(state);

            if let Some(ref prev_id) = prev_selected_id {
                if let Some(pos) = state.containers.filtered.iter().position(|&idx| {
                    state.containers.items.get(idx).map(|c| &c.id == prev_id).unwrap_or(false)
                }) {
                    state.containers.selected = pos;
                }
            }
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
            state.container_extra.stopping_containers.insert(id.clone());
            commands.push(Command::StopContainer(id.clone()));
        }
        AppEvent::ContainerStopped(id) => {
            state.container_extra.stopping_containers.remove(id);
            state.container_extra.starting_containers.remove(id);
        }
        AppEvent::ContainerStarted(id) => {
            state.container_extra.starting_containers.remove(id);
        }
        AppEvent::StartContainer(id) => commands.push(Command::StartContainer(id.clone())),
        AppEvent::ContainerDeleted(id) => {
            state.container_extra.deleting_containers.remove(id);
        }
        AppEvent::ToggleSelectionMode => {
            state.container_extra.selection_mode = !state.container_extra.selection_mode;
            if !state.container_extra.selection_mode {
                state.container_extra.selected_ids.clear();
            }
        }
        AppEvent::ToggleSelectContainer(id) if state.container_extra.selection_mode => {
            if state.container_extra.selected_ids.contains(id) {
                state.container_extra.selected_ids.remove(id);
            } else {
                state.container_extra.selected_ids.insert(id.clone());
            }
        }
        AppEvent::SelectAllContainers if state.container_extra.selection_mode => {
            for &idx in &state.containers.filtered {
                if let Some(c) = state.containers.items.get(idx) {
                    state.container_extra.selected_ids.insert(c.id.clone());
                }
            }
        }
        AppEvent::ToggleColumnPicker => {
            state.containers.show_column_picker = !state.containers.show_column_picker;
        }
        AppEvent::ToggleColumn(name) => {
            if state.containers.show_column_picker {
                let col_count = 4;
                crate::app::reducers::handle_column_nav(name, col_count, &mut state.containers.column_picker_selection);
                match name.as_str() {
                    "name" => state.config.container_columns.show_name = !state.config.container_columns.show_name,
                    "image" => state.config.container_columns.show_image = !state.config.container_columns.show_image,
                    "state" => state.config.container_columns.show_state = !state.config.container_columns.show_state,
                    "ports" => state.config.container_columns.show_ports = !state.config.container_columns.show_ports,
                    _ => {}
                }
                commands.push(Command::SaveConfig);
            }
        }
        AppEvent::CycleStatusFilter => {
            state.container_extra.status_filter = match state.container_extra.status_filter.as_str() {
                "" => "running".to_string(),
                "running" => "exited".to_string(),
                "exited" => "paused".to_string(),
                _ => String::new(),
            };
            state.containers.selected = 0;
            apply_filter(state);
        }
        AppEvent::ToggleSortDirection => {
            state.containers.sort_ascending = !state.containers.sort_ascending;
            state.containers.apply_sort();
            state.containers.reorder_by_group();
        }
        AppEvent::BatchToggleContainers(ids) => {
            for id in ids {
                let is_running = state.containers.items.iter()
                    .find(|c| c.id == *id)
                    .map(|c| c.state == "running")
                    .unwrap_or(false);
                if is_running {
                    state.container_extra.stopping_containers.insert(id.clone());
                } else {
                    state.container_extra.starting_containers.insert(id.clone());
                }
            }
            commands.push(Command::BatchToggleContainers(ids.clone()));
        }
        _ => {}
    }
    commands
}
