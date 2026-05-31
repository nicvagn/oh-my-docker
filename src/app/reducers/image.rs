use std::time::Instant;

use crate::app::event::{AppEvent, Command, ContainerOpts};
use crate::app::state::AppState;


fn apply_filter(state: &mut AppState) {
    state.images.apply_filter(|_| true);
}

pub fn reduce(state: &mut AppState, event: &AppEvent) -> Vec<Command> {
    let mut commands = Vec::new();
    match event {
        AppEvent::ImagesUpdated(images) => {
            state.images.items = images.clone();
            state.images.loading = false;
            state.images.last_updated = Some(Instant::now());
            apply_filter(state);
            state.images.apply_sort();
        }
        AppEvent::SelectImage(idx) if *idx < state.images.filtered.len() => {
            state.images.selected = *idx;
        }
        AppEvent::FilterImages(q) => {
            state.images.filter = q.clone();
            if state.images.filter.is_empty() {
                state.images.filter_active = false;
            }
            apply_filter(state);
        }
        AppEvent::ActivateImageFilter => {
            state.images.filter_active = true;
        }
        AppEvent::PrunedImages(count) => {
            state.error = Some(format!("Pruned {} unused images", count));
            state.error_timer = 10;
        }
        AppEvent::RunImage(repository, tag) => {
            let image_id = format!("{}:{}", repository, tag);
            let latest = state.config.latest_shell.clone().unwrap_or_else(|| "bash".to_string());
            let per_image = state.config.images.get(repository).cloned().unwrap_or_default();
            let shell = per_image.shell.unwrap_or(latest);
            let user = per_image.user.unwrap_or_default();
            let workdir = per_image.workdir.unwrap_or_default();
            state.navigation.image_run = Some(crate::app::state::ImageRunState {
                image_id: image_id.clone(),
                command: String::new(),
                shell,
                user,
                workdir,
                env_vars: String::new(),
                port_mapping: String::new(),
                volumes: String::new(),
                container_name: String::new(),
                autoremove: true,
                restart_policy: String::new(),
                memory_limit: String::new(),
                cpu_limit: String::new(),
                network: String::new(),
                labels: String::new(),
                privileged: false,
                field_focus: 0,
                validation_errors: Vec::new(),
                show_advanced: false,
            });
            state.navigation.mode_stack.push(crate::app::mode::Mode::ImageRun(image_id));
        }
        AppEvent::ImageRunFieldUpdate(field, value) => {
            if let Some(ref mut run) = state.navigation.image_run {
                match field {
                    crate::app::event::ImageRunField::Command => run.command = value.clone(),
                    crate::app::event::ImageRunField::Shell => run.shell = value.clone(),
                    crate::app::event::ImageRunField::User => run.user = value.clone(),
                    crate::app::event::ImageRunField::Workdir => run.workdir = value.clone(),
                    crate::app::event::ImageRunField::EnvVars => run.env_vars = value.clone(),
                    crate::app::event::ImageRunField::PortMapping => run.port_mapping = value.clone(),
                    crate::app::event::ImageRunField::Volumes => run.volumes = value.clone(),
                    crate::app::event::ImageRunField::ContainerName => run.container_name = value.clone(),
                    crate::app::event::ImageRunField::RestartPolicy => run.restart_policy = value.clone(),
                    crate::app::event::ImageRunField::MemoryLimit => run.memory_limit = value.clone(),
                    crate::app::event::ImageRunField::CpuLimit => run.cpu_limit = value.clone(),
                    crate::app::event::ImageRunField::Network => run.network = value.clone(),
                    crate::app::event::ImageRunField::Labels => run.labels = value.clone(),
                    crate::app::event::ImageRunField::Privileged => run.privileged = !run.privileged,
                }
            }
        }
        AppEvent::ImageRunToggleAutoremove => {
            if let Some(ref mut run) = state.navigation.image_run {
                run.autoremove = !run.autoremove;
            }
        }
        AppEvent::ImageRunToggleAdvanced => {
            if let Some(ref mut run) = state.navigation.image_run {
                run.show_advanced = !run.show_advanced;
                if !run.show_advanced && run.field_focus >= 9 {
                    run.field_focus = 0;
                }
            }
        }
        AppEvent::ImageRunFocusNext => {
            if let Some(ref mut run) = state.navigation.image_run {
                let max_fields = if run.show_advanced { 15 } else { 9 };
                run.field_focus = (run.field_focus + 1) % max_fields;
            }
        }
        AppEvent::ImageRunFocusPrev => {
            if let Some(ref mut run) = state.navigation.image_run {
                let max_fields = if run.show_advanced { 15 } else { 9 };
                run.field_focus = if run.field_focus == 0 {
                    max_fields - 1
                } else {
                    run.field_focus.saturating_sub(1)
                };
            }
        }
        AppEvent::ToggleColumnPicker => {
            state.images.show_column_picker = !state.images.show_column_picker;
        }
        AppEvent::ToggleColumn(name) => {
            if state.images.show_column_picker {
                let col_count = 4;
                crate::app::reducers::handle_column_nav(name, col_count, &mut state.images.column_picker_selection);
                match name.as_str() {
                    "repository" => state.config.image_columns.show_repository = !state.config.image_columns.show_repository,
                    "tag" => state.config.image_columns.show_tag = !state.config.image_columns.show_tag,
                    "id" => state.config.image_columns.show_id = !state.config.image_columns.show_id,
                    "size" => state.config.image_columns.show_size = !state.config.image_columns.show_size,
                    _ => {}
                }
                commands.push(Command::SaveConfig);
            }
        }
        AppEvent::ToggleSortDirection => {
            state.images.sort_ascending = !state.images.sort_ascending;
            state.images.apply_sort();
        }
        AppEvent::ImageRunSubmit => {
            let mut errors: Vec<(usize, String)> = Vec::new();
            if let Some(ref run) = state.navigation.image_run {
                for line in run.port_mapping.lines() {
                    let line = line.trim();
                    if line.is_empty() { continue; }
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.is_empty() || parts.len() > 3 {
                        errors.push((5, format!("Invalid port '{}': use HOST:CONTAINER or CONTAINER", line)));
                        continue;
                    }
                    let container_port = parts.last().unwrap().trim();
                    if container_port.parse::<u16>().is_err() {
                        errors.push((5, format!("Invalid container port '{}' in '{}': must be a number", container_port, line)));
                    }
                }
                for line in run.volumes.lines() {
                    let line = line.trim();
                    if line.is_empty() { continue; }
                    if !line.contains(':') {
                        errors.push((6, format!("Invalid volume '{}': must be HOST:CONTAINER[:ro|:rw]", line)));
                    }
                }
            }
            if !errors.is_empty() {
                if let Some(ref mut run) = state.navigation.image_run {
                    run.validation_errors = errors;
                }
            } else if let Some(ref run) = state.navigation.image_run {
                let image_base = crate::util::image_base_name(&run.image_id).to_string();
                let entry = state.config.images.entry(image_base.clone()).or_default();
                entry.shell = if run.shell.is_empty() { None } else { Some(run.shell.clone()) };
                entry.user = if run.user.is_empty() { None } else { Some(run.user.clone()) };
                entry.workdir = if run.workdir.is_empty() { None } else { Some(run.workdir.clone()) };
                commands.push(Command::CreateContainer(ContainerOpts {
                    image: run.image_id.clone(),
                    cmd: run.command.clone(),
                    shell: run.shell.clone(),
                    user: run.user.clone(),
                    workdir: run.workdir.clone(),
                    env_vars: run.env_vars.clone(),
                    port_mapping: run.port_mapping.clone(),
                    volumes: run.volumes.clone(),
                    name: run.container_name.clone(),
                    autoremove: run.autoremove,
                    restart_policy: run.restart_policy.clone(),
                    memory_limit: run.memory_limit.clone(),
                    cpu_limit: run.cpu_limit.clone(),
                    network: run.network.clone(),
                    labels: run.labels.clone(),
                    privileged: run.privileged,
                }));
                commands.push(Command::SaveConfig);
                state.navigation.image_run = None;
                state.navigation.mode_stack.back();
            }
        }
        _ => {}
    }
    commands
}
