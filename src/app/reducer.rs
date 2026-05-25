use crate::app::event::{AppEvent, Command, ConfirmAction, ImageRunField, ShellConfigField};
use crate::app::mode::Mode;
use crate::app::state::{AppState, DetailsState, ImageRunState, LogState, ShellConfigState, ShellState};
use crate::config::ContainerShellConfig;
use crate::search::fuzzy::Fuzzy;

fn apply_container_filter(state: &mut AppState) {
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

fn apply_image_filter(state: &mut AppState) {
    let items = &state.images.items;
    let filter = &state.images.filter;
    if filter.is_empty() {
        state.images.filtered = (0..items.len()).collect();
    } else {
        let fuzzy = Fuzzy::new();
        let results = fuzzy.filter(filter, items, |i| &i.repository);
        state.images.filtered = results.into_iter().map(|(i, _)| i).collect();
    }
    if state.images.selected >= state.images.filtered.len() {
        state.images.selected = state.images.filtered.len().saturating_sub(1);
    }
}

pub fn reduce(state: AppState, event: AppEvent) -> (AppState, Vec<Command>) {
    let mut new_state = state;
    let mut commands = Vec::new();

    match event {
        AppEvent::Quit => new_state.quit = true,

        AppEvent::Navigate(mode @ Mode::Shell(_)) => {
            if let Mode::Shell(id) = &mode {
                new_state.shell = Some(ShellState {
                    container_id: id.clone(),
                    active: true,
                    stop_on_exit: false,
                    shell: "bash".to_string(),
                    user: String::new(),
                    workdir: String::new(),
                });
            }
            new_state.mode_stack.push(mode);
        }
          AppEvent::Navigate(mode @ Mode::Logs(_)) => {
            if let Mode::Logs(id) = &mode {
                // Cancel any existing log stream for the current container
                if let Some(ref logs) = new_state.logs {
                    if let Some(handle) = new_state.log_streams.remove(&logs.container_id) {
                        handle.abort();
                    }
                }
                new_state.logs = Some(LogState {
                    container_id: id.clone(),
                    buffer: Vec::new(),
                    max_lines: 10000,
                    paused: false,
                    search: String::new(),
                    search_active: false,
                    scroll_offset: 0,
                    tail: true,
                });
                commands.push(Command::FetchLogs(id.clone()));
            }
            new_state.mode_stack.push(mode);
        }
        AppEvent::Navigate(mode @ Mode::ShellConfig(_)) => {
            if let Mode::ShellConfig(id) = &mode {
                let container = new_state.containers.items.iter().find(|c| c.id == *id);
                let name = container.map(|c| c.name.clone()).unwrap_or_default();
                let image_base = container.map(|c| crate::util::image_base_name(&c.image).to_string());
                let latest = new_state.config.latest_shell.clone().unwrap_or_else(|| "bash".to_string());
                let per_container = new_state.config.containers.get(&name).cloned().unwrap_or_default();
                let per_image = image_base.as_ref()
                    .and_then(|ib| new_state.config.images.get(ib).cloned())
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
                new_state.shell_config = Some(ShellConfigState {
                    container_id: id.clone(),
                    shell,
                    user,
                    workdir,
                    field_focus: 0,
                });
            }
            new_state.mode_stack.push(mode);
        }
        AppEvent::Navigate(mode) => new_state.mode_stack.push(mode),
  AppEvent::Back => {
            if let Some(ref logs) = new_state.logs {
                if let Some(handle) = new_state.log_streams.remove(&logs.container_id) {
                    handle.abort();
                }
            }
            new_state.logs = None;
            new_state.shell_config = None;
            new_state.image_run = None;
            new_state.mode_stack.back();
        }

        AppEvent::ShowHelp => new_state.mode_stack.push(Mode::Help),
        AppEvent::HideHelp => {
            if *new_state.mode_stack.current() == Mode::Help {
                new_state.mode_stack.back();
            }
        }
        AppEvent::ScrollHelp(delta) => {
            if delta > 0 {
                new_state.help.scroll_offset = new_state.help.scroll_offset.saturating_add(delta as usize);
            } else {
                new_state.help.scroll_offset = new_state.help.scroll_offset.saturating_sub((-delta) as usize);
            }
            new_state.help.scroll_offset = new_state.help.scroll_offset.min(10000);
        }

        AppEvent::Tick => {
            if new_state.error_timer > 0 {
                new_state.error_timer -= 1;
                if new_state.error_timer == 0 {
                    new_state.error = None;
                }
            }
        }

        AppEvent::CheckUpdate => {
            if let Some((version, url)) = new_state.update_available.take() {
                commands.push(Command::DownloadUpdate { version, download_url: url });
                new_state.error = Some("Downloading update...".to_string());
                new_state.error_timer = 5;
            } else {
                commands.push(Command::CheckUpdate);
                new_state.error = Some("Checking for updates...".to_string());
                new_state.error_timer = 5;
            }
        }
        AppEvent::UpdateAvailable(version, url) => {
            new_state.update_available = Some((version, url));
        }
        AppEvent::UpdateNotAvailable => {
            new_state.error = Some("Already up to date".to_string());
            new_state.error_timer = 3;
        }

        AppEvent::Error(msg) => {
            new_state.error = Some(msg);
            new_state.error_timer = 5;
        }

        AppEvent::ShowConfirmDialog(prompt, action) => {
            new_state.mode_stack.push(Mode::ConfirmDialog { prompt, action });
        }
        AppEvent::ConfirmYes => {
            if let Mode::ConfirmDialog { action, .. } = new_state.mode_stack.current() {
                let action = action.clone();
                new_state.mode_stack.back();
                match action {
                    ConfirmAction::DeleteContainer(id) => {
                        new_state.containers.deleting_containers.insert(id.clone());
                        commands.push(Command::DeleteContainer(id));
                    }
                    ConfirmAction::RemoveImage(id) => {
                        commands.push(Command::RemoveImage(id));
                    }
                    ConfirmAction::RemoveNetwork(id) => {
                        commands.push(Command::RemoveNetwork(id));
                    }
                    ConfirmAction::RemoveVolume(name) => {
                        commands.push(Command::RemoveVolume(name));
                    }
                }
            }
        }
        AppEvent::ConfirmNo => {
            if matches!(new_state.mode_stack.current(), Mode::ConfirmDialog { .. }) {
                new_state.mode_stack.back();
            }
        }

        // --- Containers ---
        AppEvent::ContainersUpdated(containers) => {
            new_state.containers.items = containers;
            new_state.containers.loading = false;
            new_state.containers.docker_connected = true;
            apply_container_filter(&mut new_state);
        }
        AppEvent::SelectContainer(idx) => {
            if idx < new_state.containers.filtered.len() {
                new_state.containers.selected = idx;
            }
        }
        AppEvent::FilterContainers(q) => {
            new_state.containers.filter = q;
            if new_state.containers.filter.is_empty() {
                new_state.containers.filter_active = false;
            }
            apply_container_filter(&mut new_state);
        }
        AppEvent::ActivateFilter => {
            new_state.containers.filter_active = true;
        }
        AppEvent::FilterSubmit(sel) => {
            match new_state.mode_stack.current() {
                Mode::Containers => {
                    new_state.containers.filter_active = false;
                    if let Some(idx) = sel {
                        if idx < new_state.containers.filtered.len() {
                            new_state.containers.selected = idx;
                        }
                    }
                }
                Mode::Images => {
                    new_state.images.filter_active = false;
                    if let Some(idx) = sel {
                        if idx < new_state.images.filtered.len() {
                            new_state.images.selected = idx;
                        }
                    }
                }
                _ => {}
            }
        }
        AppEvent::ActivateImageFilter => {
            new_state.images.filter_active = true;
        }

        AppEvent::ShowDetails => {
            if let Some(&idx) = new_state.containers.filtered.get(new_state.containers.selected) {
                if let Some(container) = new_state.containers.items.get(idx) {
                    let id = container.id.clone();
                    new_state.mode_stack.push(Mode::ContainerDetails(id.clone()));
                    new_state.details = Some(DetailsState {
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
            let prev = new_state.details.take();
            let existing_id = prev.as_ref().map(|d| d.id.clone()).unwrap_or_default();
            let existing_scroll = prev.as_ref().map(|d| d.scroll_offset).unwrap_or(0);
            new_state.details = Some(DetailsState {
                id: existing_id,
                container_id: name,
                json: Some(serde_json::to_string_pretty(&json).unwrap_or_default()),
                scroll_offset: existing_scroll,
            });
        }

        AppEvent::RestartContainer(id) => commands.push(Command::RestartContainer(id)),
       AppEvent::StopContainer(id) => {
            new_state.containers.stopping_containers.insert(id.clone());
            commands.push(Command::StopContainer(id));
        }
        AppEvent::ContainerStopped(id) => {
            new_state.containers.stopping_containers.remove(&id);
        }
        AppEvent::StartContainer(id) => commands.push(Command::StartContainer(id)),
        AppEvent::DeleteContainer(id) => {
            new_state.containers.deleting_containers.insert(id.clone());
            commands.push(Command::DeleteContainer(id));
        }
        AppEvent::ContainerDeleted(id) => {
            new_state.containers.deleting_containers.remove(&id);
        }
        AppEvent::ScrollDetails(delta) => {
            if let Some(ref mut d) = new_state.details {
                if delta > 0 {
                    d.scroll_offset = d.scroll_offset.saturating_add(delta as usize);
                } else {
                    d.scroll_offset = d.scroll_offset.saturating_sub((-delta) as usize);
                }
                d.scroll_offset = d.scroll_offset.min(10000);
            }
        }

        // --- Logs ---
        AppEvent::LogLines(id, lines) => {
            let should_swap = match new_state.logs {
                Some(ref l) => l.container_id != id,
                None => true,
            };
            if should_swap {
                new_state.logs = Some(LogState {
                    container_id: id.clone(),
                    buffer: Vec::new(),
                    max_lines: 10000,
                    paused: false,
                    search: String::new(),
                    search_active: false,
                    scroll_offset: 0,
                    tail: true,
                });
            }
            if let Some(ref mut log_state) = new_state.logs {
                let n = lines.len();
                for line in lines {
                    log_state.buffer.push(line);
                }
                // When paused, keep the view frozen by compensating scroll_offset
                // for new lines added to the buffer
                if log_state.paused {
                    log_state.scroll_offset = log_state.scroll_offset.saturating_add(n);
                }
                if log_state.buffer.len() > log_state.max_lines {
                    let excess = log_state.buffer.len() - log_state.max_lines;
                    log_state.buffer.drain(0..excess);
                }
                if log_state.tail {
                    log_state.scroll_offset = 0;
                }
            }
        }

        // --- Images ---
        AppEvent::ImagesUpdated(images) => {
            new_state.images.items = images;
            new_state.images.loading = false;
            apply_image_filter(&mut new_state);
        }
        AppEvent::SelectImage(idx) => {
            if idx < new_state.images.filtered.len() {
                new_state.images.selected = idx;
            }
        }
        AppEvent::FilterImages(q) => {
            new_state.images.filter = q;
            if new_state.images.filter.is_empty() {
                new_state.images.filter_active = false;
            }
            apply_image_filter(&mut new_state);
        }
        AppEvent::RemoveImage(id) => commands.push(Command::RemoveImage(id)),
          AppEvent::RunImage(id) => {
            new_state.image_run = Some(ImageRunState {
                image_id: id.clone(),
                command: String::new(),
                shell: "sh".to_string(),
                user: String::new(),
                workdir: String::new(),
                env_vars: String::new(),
                port_mapping: String::new(),
                volumes: String::new(),
                container_name: String::new(),
                autoremove: true,
                field_focus: 0,
            });
            new_state.mode_stack.push(Mode::ImageRun(id));
        }

        AppEvent::TogglePause => {
            if let Some(ref mut log) = new_state.logs {
                log.paused = !log.paused;
                if log.paused {
                    log.tail = false;
                } else {
                    log.tail = true;
                    log.scroll_offset = 0;
                }
            }
        }
        AppEvent::ActivateLogSearch => {
            if let Some(ref mut log) = new_state.logs {
                log.search_active = true;
            }
        }
        AppEvent::SearchLogs(q) => {
            if let Some(ref mut log) = new_state.logs {
                log.search = q.clone();
                log.search_active = !q.is_empty();
            }
        }
        AppEvent::SubmitLogSearch => {
            if let Some(ref mut log) = new_state.logs {
                log.search_active = false;
            }
        }
        AppEvent::ScrollLogs(delta) => {
            if let Some(ref mut log) = new_state.logs {
                if delta > 0 {
                    log.scroll_offset = log.scroll_offset.saturating_add(delta as usize);
                } else {
                    log.scroll_offset = log.scroll_offset.saturating_sub((-delta) as usize);
                }
                log.tail = log.scroll_offset == 0;
            }
        }
        AppEvent::JumpTop => {
            if let Some(ref mut log) = new_state.logs {
                log.scroll_offset = log.buffer.len();
                log.tail = false;
            }
        }
        AppEvent::JumpBottom => {
            if let Some(ref mut log) = new_state.logs {
                log.scroll_offset = 0;
                log.tail = true;
            }
        }

        // --- Events ---
        AppEvent::EventsUpdated(events) => {
            for e in events {
                new_state.events.buffer.push(e);
            }
            if new_state.events.buffer.len() > new_state.events.max_events {
                let excess = new_state.events.buffer.len() - new_state.events.max_events;
                new_state.events.buffer.drain(0..excess);
            }
        }
          AppEvent::CloseShell => {
            let shell_data = new_state.shell.take();
            new_state.mode_stack.back();
            if let Some(s) = shell_data {
                if s.stop_on_exit {
                    new_state.containers.stopping_containers.insert(s.container_id.clone());
                    commands.push(Command::StopContainer(s.container_id));
                }
            }
        }
        AppEvent::StartShell(id, shell, user, workdir) => {
            new_state.shell = Some(ShellState {
                container_id: id.clone(),
                active: true,
                stop_on_exit: true,
                shell,
                user,
                workdir,
            });
            new_state.mode_stack.push(Mode::Shell(id));
        }
        AppEvent::ImageRunSubmit => {
            if let Some(ref run) = new_state.image_run {
                commands.push(Command::CreateContainer(crate::app::event::ContainerOpts {
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
                }));
            }
            new_state.image_run = None;
            new_state.mode_stack.back();
        }
        AppEvent::ImageRunFieldUpdate(field, value) => {
            if let Some(ref mut run) = new_state.image_run {
                match field {
                    ImageRunField::Command => run.command = value,
                    ImageRunField::Shell => run.shell = value,
                    ImageRunField::User => run.user = value,
                    ImageRunField::Workdir => run.workdir = value,
                    ImageRunField::EnvVars => run.env_vars = value,
                    ImageRunField::PortMapping => run.port_mapping = value,
                    ImageRunField::Volumes => run.volumes = value,
                    ImageRunField::ContainerName => run.container_name = value,
                }
            }
        }
          AppEvent::ImageRunToggleAutoremove => {
            if let Some(ref mut run) = new_state.image_run {
                run.autoremove = !run.autoremove;
            }
        }
         AppEvent::ImageRunFocusNext => {
            if let Some(ref mut run) = new_state.image_run {
                run.field_focus = (run.field_focus + 1) % 9;
            }
        }
        AppEvent::ImageRunFocusPrev => {
            if let Some(ref mut run) = new_state.image_run {
                run.field_focus = if run.field_focus == 0 {
                    8
                } else {
                    run.field_focus.saturating_sub(1)
                };
            }
        }
        AppEvent::ShellConfigSubmit => {
            if let Some(config) = new_state.shell_config.take() {
                let id = config.container_id.clone();
                let name = new_state.containers.items.iter()
                    .find(|c| c.id == id)
                    .map(|c| c.name.clone())
                    .unwrap_or(id.clone());
                new_state.shell = Some(ShellState {
                    container_id: id.clone(),
                    active: true,
                    stop_on_exit: false,
                    shell: config.shell.clone(),
                    user: config.user.clone(),
                    workdir: config.workdir.clone(),
                });
                new_state.config.latest_shell = Some(config.shell.clone());
                new_state.config.containers.insert(name, ContainerShellConfig {
                    shell: Some(config.shell.clone()),
                    user: if config.user.is_empty() { None } else { Some(config.user.clone()) },
                    workdir: if config.workdir.is_empty() { None } else { Some(config.workdir.clone()) },
                });
                if let Some(container) = new_state.containers.items.iter().find(|c| c.id == id) {
                    let img_base = crate::util::image_base_name(&container.image).to_string();
                    let entry = new_state.config.images.entry(img_base).or_default();
                    entry.shell = Some(config.shell.clone());
                    entry.user = if config.user.is_empty() { None } else { Some(config.user.clone()) };
                    entry.workdir = if config.workdir.is_empty() { None } else { Some(config.workdir.clone()) };
                }
                commands.push(Command::SaveConfig);
                new_state.mode_stack.back();
                new_state.mode_stack.push(Mode::Shell(id));
            }
        }
        AppEvent::ShellConfigFieldUpdate(field, value) => {
            if let Some(ref mut cfg) = new_state.shell_config {
                match field {
                    ShellConfigField::Shell => cfg.shell = value,
                    ShellConfigField::User => cfg.user = value,
                    ShellConfigField::Workdir => cfg.workdir = value,
                }
            }
        }
        AppEvent::ShellConfigFocusNext => {
            if let Some(ref mut cfg) = new_state.shell_config {
                cfg.field_focus = (cfg.field_focus + 1) % 3;
            }
        }
        AppEvent::ShellConfigFocusPrev => {
            if let Some(ref mut cfg) = new_state.shell_config {
                cfg.field_focus = (cfg.field_focus + 2) % 3;
            }
        }
        AppEvent::FilterEvents(q) => {
            new_state.events.filter = q;
            if new_state.events.filter.is_empty() {
                new_state.events.filter_active = false;
            }
        }
        AppEvent::ActivateEventsFilter => {
            new_state.events.filter_active = true;
        }
        AppEvent::EventsFilterSubmit => {
            new_state.events.filter_active = false;
        }

        // --- Statistics ---
        AppEvent::StatisticsUpdated(stats) => {
            new_state.statistics.items = stats;
            new_state.statistics.loading = false;
        }

        // --- Networks ---
        AppEvent::NetworksUpdated(networks) => {
            new_state.networks.items = networks;
            new_state.networks.loading = false;
        }
        AppEvent::SelectNetwork(idx) => {
            if idx < new_state.networks.items.len() {
                new_state.networks.selected = idx;
            }
        }
        AppEvent::RemoveNetwork(id) => commands.push(Command::RemoveNetwork(id)),

        // --- Volumes ---
        AppEvent::VolumesUpdated(volumes) => {
            new_state.volumes.items = volumes;
            new_state.volumes.loading = false;
        }
        AppEvent::SelectVolume(idx) => {
            if idx < new_state.volumes.items.len() {
                new_state.volumes.selected = idx;
            }
        }
        AppEvent::RemoveVolume(name) => commands.push(Command::RemoveVolume(name)),
    }

    (new_state, commands)
}
