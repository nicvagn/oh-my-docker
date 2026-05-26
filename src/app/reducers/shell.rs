use crate::app::event::{AppEvent, Command};
use crate::app::state::AppState;
use crate::config::ContainerShellConfig;

pub fn reduce(state: &mut AppState, event: &AppEvent) -> Vec<Command> {
    let mut commands = Vec::new();
    match event {
        AppEvent::CloseShell => {
            let shell_data = state.navigation.shell.take();
            state.navigation.mode_stack.back();
            if let Some(s) = shell_data {
                if s.stop_on_exit {
                    state.containers.stopping_containers.insert(s.container_id.clone());
                    commands.push(Command::StopContainer(s.container_id));
                }
            }
        }
        AppEvent::StartShell(id, shell, user, workdir) => {
            state.navigation.shell = Some(crate::app::state::ShellState {
                container_id: id.clone(),
                active: true,
                stop_on_exit: true,
                shell: shell.clone(),
                user: user.clone(),
                workdir: workdir.clone(),
            });
            state.navigation.mode_stack.push(crate::app::mode::Mode::Shell(id.clone()));
        }
        AppEvent::ShellConfigSubmit => {
            if let Some(config) = state.navigation.shell_config.take() {
                let id = config.container_id.clone();
                let name = state.containers.items.iter()
                    .find(|c| c.id == id)
                    .map(|c| c.name.clone())
                    .unwrap_or(id.clone());
                state.navigation.shell = Some(crate::app::state::ShellState {
                    container_id: id.clone(),
                    active: true,
                    stop_on_exit: false,
                    shell: config.shell.clone(),
                    user: config.user.clone(),
                    workdir: config.workdir.clone(),
                });
                state.config.latest_shell = Some(config.shell.clone());
                state.config.containers.insert(name, ContainerShellConfig {
                    shell: Some(config.shell.clone()),
                    user: if config.user.is_empty() { None } else { Some(config.user.clone()) },
                    workdir: if config.workdir.is_empty() { None } else { Some(config.workdir.clone()) },
                });
                if let Some(container) = state.containers.items.iter().find(|c| c.id == id) {
                    let img_base = crate::util::image_base_name(&container.image).to_string();
                    let entry = state.config.images.entry(img_base).or_default();
                    entry.shell = Some(config.shell.clone());
                    entry.user = if config.user.is_empty() { None } else { Some(config.user.clone()) };
                    entry.workdir = if config.workdir.is_empty() { None } else { Some(config.workdir.clone()) };
                }
                commands.push(Command::SaveConfig);
                state.navigation.mode_stack.back();
                state.navigation.mode_stack.push(crate::app::mode::Mode::Shell(id));
            }
        }
        AppEvent::ShellConfigFieldUpdate(field, value) => {
            if let Some(ref mut cfg) = state.navigation.shell_config {
                match field {
                    crate::app::event::ShellConfigField::Shell => cfg.shell = value.clone(),
                    crate::app::event::ShellConfigField::User => cfg.user = value.clone(),
                    crate::app::event::ShellConfigField::Workdir => cfg.workdir = value.clone(),
                }
            }
        }
        AppEvent::ShellConfigFocusNext => {
            if let Some(ref mut cfg) = state.navigation.shell_config {
                cfg.field_focus = (cfg.field_focus + 1) % 3;
            }
        }
        AppEvent::ShellConfigFocusPrev => {
            if let Some(ref mut cfg) = state.navigation.shell_config {
                cfg.field_focus = (cfg.field_focus + 2) % 3;
            }
        }
        _ => {}
    }
    commands
}
