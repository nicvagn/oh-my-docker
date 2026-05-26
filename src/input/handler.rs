use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crate::app::event::{AppEvent, ConfirmAction, ImageRunField, ShellConfigField};
use crate::app::mode::Mode;
use crate::app::state::AppState;
use crate::input::keys;

fn handle_containers_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
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
                        Some(AppEvent::ShowConfirmDialog(
                            format!("Stop {} selected container(s)?", ids.len()),
                            ConfirmAction::BatchStopContainers,
                        ))
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
            _ => None,
        }
    }
}

fn handle_details_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    match key.code {
        KeyCode::Char('l') => state.details.as_ref().map(|d| AppEvent::Navigate(Mode::Logs(d.id.clone()))),
        KeyCode::Char('s') => state.details.as_ref().map(|d| AppEvent::Navigate(Mode::ShellConfig(d.id.clone()))),
        KeyCode::Char('r') => state.details.as_ref().map(|d| AppEvent::RestartContainer(d.id.clone())),
        KeyCode::Char('S') => state.details.as_ref().map(|d| {
            let cid = d.id.clone();
            let container = state.containers.items.iter().find(|c| c.id == d.id);
            match container.map(|c| c.state.as_str()) {
                Some("running") => AppEvent::StopContainer(cid),
                _ => AppEvent::StartContainer(cid),
            }
        }),
        KeyCode::Up | KeyCode::Char('k') => Some(AppEvent::ScrollDetails(-1)),
        KeyCode::Down | KeyCode::Char('j') => Some(AppEvent::ScrollDetails(1)),
        KeyCode::PageUp => Some(AppEvent::ScrollDetails(-20)),
        KeyCode::PageDown => Some(AppEvent::ScrollDetails(20)),
        KeyCode::Char('g') => Some(AppEvent::ScrollDetails(10000)),
        KeyCode::Char('G') => Some(AppEvent::ScrollDetails(-10000)),
        _ => None,
    }
}

fn handle_logs_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    let search_active = state.logs.as_ref().map(|l| l.search_active).unwrap_or(false);
    if search_active {
        match (key.code, key.modifiers) {
            (KeyCode::Backspace, _) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                state.logs.as_ref().map(|l| {
                    let new_q = l.search.chars().take(l.search.chars().count().saturating_sub(1)).collect();
                    AppEvent::SearchLogs(new_q)
                })
            }
            (KeyCode::Esc, _) => Some(AppEvent::SearchLogs(String::new())),
            (KeyCode::Enter, _) => Some(AppEvent::SubmitLogSearch),
            (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                state.logs.as_ref().map(|l| {
                    let new_q = format!("{}{}", l.search, c);
                    AppEvent::SearchLogs(new_q)
                })
            }
            _ => None,
        }
    } else {
        match key.code {
            KeyCode::Char(' ') | KeyCode::Char('p') => Some(AppEvent::TogglePause),
                KeyCode::Char('r') => {
                state.logs.as_ref().and_then(|l| {
                    if l.paused { Some(AppEvent::TogglePause) } else { None }
                })
            }
            KeyCode::Char('/') => Some(AppEvent::ActivateLogSearch),
            KeyCode::Char('g') => Some(AppEvent::JumpTop),
            KeyCode::Char('G') => Some(AppEvent::JumpBottom),
            KeyCode::Char('s') => state.logs.as_ref().map(|l| AppEvent::ExportLogs(l.container_id.clone())),
            KeyCode::Char('T') => Some(AppEvent::ToggleLogTimestamps),
            KeyCode::Up | KeyCode::Char('k') => Some(AppEvent::ScrollLogs(1)),
            KeyCode::Down | KeyCode::Char('j') => Some(AppEvent::ScrollLogs(-1)),
            KeyCode::PageUp => Some(AppEvent::ScrollLogs(20)),
            KeyCode::PageDown => Some(AppEvent::ScrollLogs(-20)),
            _ => None,
        }
    }
}

fn handle_images_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    if state.images.filter_active {
        match (key.code, key.modifiers) {
            (KeyCode::Backspace, _) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                let new_q = state.images.filter.chars().take(state.images.filter.chars().count().saturating_sub(1)).collect();
                Some(AppEvent::FilterImages(new_q))
            }
            (KeyCode::Esc, _) => Some(AppEvent::FilterImages(String::new())),
            (KeyCode::Enter, _) => Some(AppEvent::FilterSubmit(None)),
            (KeyCode::Down, _) => {
                let next = (state.images.selected + 1).min(state.images.filtered.len().saturating_sub(1));
                Some(AppEvent::FilterSubmit(Some(next)))
            }
            (KeyCode::Up, _) => {
                let prev = state.images.selected.saturating_sub(1);
                Some(AppEvent::FilterSubmit(Some(prev)))
            }
            (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                let new_q = format!("{}{}", state.images.filter, c);
                Some(AppEvent::FilterImages(new_q))
            }
            _ => None,
        }
    } else {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                let next = (state.images.selected + 1).min(state.images.filtered.len().saturating_sub(1));
                Some(AppEvent::SelectImage(next))
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let prev = state.images.selected.saturating_sub(1);
                Some(AppEvent::SelectImage(prev))
            }
            KeyCode::Char('r') => {
                state.images.filtered.get(state.images.selected)
                    .and_then(|&idx| state.images.items.get(idx))
                    .map(|img| AppEvent::RunImage(img.repository.clone(), img.tag.clone()))
            }
            KeyCode::Char('d') => {
                state.images.filtered.get(state.images.selected)
                    .and_then(|&idx| state.images.items.get(idx))
                    .map(|img| AppEvent::ShowConfirmDialog(
                        format!("Remove image '{}:{}'?", img.repository, img.tag),
                        ConfirmAction::RemoveImage(img.id.clone()),
                    ))
            }
             KeyCode::Char('D') => Some(AppEvent::ShowConfirmDialog(
                "Remove all dangling (<none>) images?".to_string(),
                ConfirmAction::RemoveDanglingImages,
            )),
            KeyCode::Char('p') => Some(AppEvent::ShowConfirmDialog(
                "Prune all unused images?".to_string(),
                ConfirmAction::PruneUnusedImages,
            )),
            KeyCode::Char('/') => Some(AppEvent::ActivateImageFilter),
            KeyCode::Enter => {
                state.images.filtered.get(state.images.selected)
                    .and_then(|&idx| state.images.items.get(idx))
                    .map(|img| AppEvent::RunImage(img.repository.clone(), img.tag.clone()))
            }
            _ => None,
        }
    }
}

fn handle_shell_key(key: KeyEvent) -> Option<AppEvent> {
    match key.code {
        KeyCode::Esc => Some(AppEvent::CloseShell),
        _ => None,
    }
}

fn handle_shell_config_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    match (key.code, key.modifiers) {
        (KeyCode::Backspace, _) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => state.shell_config.as_ref().map(|cfg| {
            let (val, field) = match cfg.field_focus {
                0 => (cfg.shell.as_str(), ShellConfigField::Shell),
                1 => (cfg.user.as_str(), ShellConfigField::User),
                _ => (cfg.workdir.as_str(), ShellConfigField::Workdir),
            };
            let new_val: String = val.chars().take(val.chars().count().saturating_sub(1)).collect();
            AppEvent::ShellConfigFieldUpdate(field, new_val)
        }),
        (KeyCode::Esc, _) => Some(AppEvent::Back),
        (KeyCode::Enter, _) => Some(AppEvent::ShellConfigSubmit),
        (KeyCode::Tab, _) | (KeyCode::Down, _) => Some(AppEvent::ShellConfigFocusNext),
        (KeyCode::Up, _) => Some(AppEvent::ShellConfigFocusPrev),
        (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => state.shell_config.as_ref().map(|cfg| {
            let (val, field) = match cfg.field_focus {
                0 => (cfg.shell.as_str(), ShellConfigField::Shell),
                1 => (cfg.user.as_str(), ShellConfigField::User),
                _ => (cfg.workdir.as_str(), ShellConfigField::Workdir),
            };
            AppEvent::ShellConfigFieldUpdate(field, format!("{}{}", val, c))
        }),
        _ => None,
    }
}

fn handle_image_run_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    match (key.code, key.modifiers) {
        (KeyCode::Backspace, _) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => state.image_run.as_ref().map(|run| {
            let (val, field) = match run.field_focus {
                0 => (run.command.as_str(), ImageRunField::Command),
                1 => (run.shell.as_str(), ImageRunField::Shell),
                2 => (run.user.as_str(), ImageRunField::User),
                3 => (run.workdir.as_str(), ImageRunField::Workdir),
                4 => (run.env_vars.as_str(), ImageRunField::EnvVars),
                5 => (run.port_mapping.as_str(), ImageRunField::PortMapping),
                6 => (run.volumes.as_str(), ImageRunField::Volumes),
                _ => (run.container_name.as_str(), ImageRunField::ContainerName),
            };
            let new_val: String = val.chars().take(val.chars().count().saturating_sub(1)).collect();
            AppEvent::ImageRunFieldUpdate(field, new_val)
        }),
        (KeyCode::Esc, _) => Some(AppEvent::Back),
        (KeyCode::Enter, _) => Some(AppEvent::ImageRunSubmit),
        (KeyCode::Tab, _) | (KeyCode::Down, _) => Some(AppEvent::ImageRunFocusNext),
        (KeyCode::Up, _) => Some(AppEvent::ImageRunFocusPrev),
        (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => state.image_run.as_ref().and_then(|run| {
            if c == 'a' && run.field_focus == 8 {
                return Some(AppEvent::ImageRunToggleAutoremove);
            }
            let (val, field) = match run.field_focus {
                0 => (run.command.as_str(), ImageRunField::Command),
                1 => (run.shell.as_str(), ImageRunField::Shell),
                2 => (run.user.as_str(), ImageRunField::User),
                3 => (run.workdir.as_str(), ImageRunField::Workdir),
                4 => (run.env_vars.as_str(), ImageRunField::EnvVars),
                5 => (run.port_mapping.as_str(), ImageRunField::PortMapping),
                6 => (run.volumes.as_str(), ImageRunField::Volumes),
                _ => (run.container_name.as_str(), ImageRunField::ContainerName),
            };
            Some(AppEvent::ImageRunFieldUpdate(field, format!("{}{}", val, c)))
        }),
        _ => None,
    }
}

fn handle_events_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    if state.events.filter_active {
        match (key.code, key.modifiers) {
            (KeyCode::Backspace, _) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                let new_q = state.events.filter.chars().take(state.events.filter.chars().count().saturating_sub(1)).collect();
                Some(AppEvent::FilterEvents(new_q))
            }
            (KeyCode::Esc, _) => Some(AppEvent::FilterEvents(String::new())),
            (KeyCode::Enter, _) => Some(AppEvent::EventsFilterSubmit),
            (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                let new_q = format!("{}{}", state.events.filter, c);
                Some(AppEvent::FilterEvents(new_q))
            }
            _ => None,
        }
    } else {
        match key.code {
            KeyCode::Char(' ') => Some(AppEvent::ToggleEventsPause),
            KeyCode::Char('e') => Some(AppEvent::ExportEvents),
            KeyCode::Char('g') => {
                if state.events.scroll_offset == 0 {
                    Some(AppEvent::ScrollEvents(10000))
                } else {
                    None
                }
            }
            KeyCode::Char('G') => Some(AppEvent::ScrollEvents(-10000)),
            KeyCode::Char('j') | KeyCode::Down => Some(AppEvent::ScrollEvents(1)),
            KeyCode::Char('k') | KeyCode::Up => Some(AppEvent::ScrollEvents(-1)),
            KeyCode::PageUp => Some(AppEvent::ScrollEvents(-10)),
            KeyCode::PageDown => Some(AppEvent::ScrollEvents(10)),
            KeyCode::Char('/') => Some(AppEvent::ActivateEventsFilter),
            _ => None,
        }
    }
}

fn handle_statistics_key(key: KeyEvent) -> Option<AppEvent> {
    match key.code {
        KeyCode::Esc => Some(AppEvent::Back),
        KeyCode::Char('t') => Some(AppEvent::ToggleSortDirection),
        KeyCode::Left => Some(AppEvent::CycleSortStat),
        KeyCode::Right => Some(AppEvent::CycleSortStat),
        _ => None,
    }
}

fn handle_networks_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    match key.code {
        KeyCode::Esc => Some(AppEvent::Back),
        KeyCode::Char('j') | KeyCode::Down => {
            let next = (state.networks.selected + 1).min(state.networks.items.len().saturating_sub(1));
            Some(AppEvent::SelectNetwork(next))
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let prev = state.networks.selected.saturating_sub(1);
            Some(AppEvent::SelectNetwork(prev))
        }
        KeyCode::Char('d') => {
            state.networks.items.get(state.networks.selected)
                .map(|n| AppEvent::ShowConfirmDialog(
                    format!("Remove network '{}'?", n.name),
                    ConfirmAction::RemoveNetwork(n.id.clone()),
                ))
        }
        _ => None,
    }
}

fn handle_volumes_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    match key.code {
        KeyCode::Esc => Some(AppEvent::Back),
        KeyCode::Char('j') | KeyCode::Down => {
            let next = (state.volumes.selected + 1).min(state.volumes.items.len().saturating_sub(1));
            Some(AppEvent::SelectVolume(next))
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let prev = state.volumes.selected.saturating_sub(1);
            Some(AppEvent::SelectVolume(prev))
        }
        KeyCode::Char('d') => {
            state.volumes.items.get(state.volumes.selected)
                .map(|v| AppEvent::ShowConfirmDialog(
                    format!("Remove volume '{}'?", v.name),
                    ConfirmAction::RemoveVolume(v.name.clone()),
                ))
        }
        _ => None,
    }
}

fn handle_confirm_dialog_key(key: KeyEvent) -> Option<AppEvent> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => Some(AppEvent::ConfirmYes),
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Some(AppEvent::ConfirmNo),
        _ => None,
    }
}

fn handle_help_key(key: KeyEvent) -> Option<AppEvent> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('?') => Some(AppEvent::HideHelp),
        KeyCode::Char('j') | KeyCode::Down | KeyCode::PageDown => Some(AppEvent::ScrollHelp(1)),
        KeyCode::Char('k') | KeyCode::Up | KeyCode::PageUp => Some(AppEvent::ScrollHelp(-1)),
        KeyCode::Char('g') => Some(AppEvent::ScrollHelp(10000)),
        KeyCode::Char('G') => Some(AppEvent::ScrollHelp(-10000)),
        _ => None,
    }
}

pub fn handle_key(key: KeyEvent, state: &AppState) -> Option<AppEvent> {
    if key.kind != KeyEventKind::Press && key.kind != KeyEventKind::Repeat {
        return None;
    }

    // Check if log search is active — intercept Esc to dismiss search instead of navigating back
    if let Mode::Logs(_) = state.mode_stack.current() {
        if let Some(ref log) = state.logs {
            if log.search_active && key.code == KeyCode::Esc {
                return Some(AppEvent::SearchLogs(String::new()));
            }
        }
    }

    if state.error_persistent {
        return Some(AppEvent::Info(String::new()));
    }

    if key.code == KeyCode::Char('U') {
        return Some(AppEvent::CheckUpdate);
    }

    let in_input_mode = state.containers.filter_active
        || state.images.filter_active
        || state.events.filter_active
        || state.logs.as_ref().map(|l| l.search_active).unwrap_or(false)
        || state.shell_config.is_some()
        || state.image_run.is_some();

    if key.code != KeyCode::Char('q') || !in_input_mode {
        if let Some(action) = keys::global_action(key.code) {
            match action {
                keys::Action::Quit => return Some(AppEvent::Quit),
                keys::Action::Back => {
                    if *state.mode_stack.current() == Mode::Help {
                        return Some(AppEvent::HideHelp);
                    }
                    if state.mode_stack.len() > 1 {
                        return Some(AppEvent::Back);
                    }
                    return None;
                }
                keys::Action::ShowHelp => return Some(AppEvent::ShowHelp),
            }
        }
    }

    match state.mode_stack.current() {
        Mode::Containers => handle_containers_key(key, state),
        Mode::ContainerDetails(_) => handle_details_key(key, state),
        Mode::Logs(_) => handle_logs_key(key, state),
        Mode::Images => handle_images_key(key, state),
        Mode::ImageRun(_) => handle_image_run_key(key, state),
        Mode::ShellConfig(_) => handle_shell_config_key(key, state),
        Mode::Shell(_) => handle_shell_key(key),
        Mode::Events => handle_events_key(key, state),
        Mode::Statistics => handle_statistics_key(key),
        Mode::Networks => handle_networks_key(key, state),
        Mode::Volumes => handle_volumes_key(key, state),
        Mode::Help => handle_help_key(key),
        Mode::ConfirmDialog { .. } => handle_confirm_dialog_key(key),
    }
}
