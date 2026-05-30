use crate::app::event::{AppEvent, Command};
use crate::app::state::AppState;

pub fn reduce(state: &mut AppState, event: AppEvent) -> Vec<Command> {
    let mut commands = Vec::new();

    match &event {
        // Global events handled inline
        AppEvent::Quit => state.quit = true,

        AppEvent::Tick => {
            state.tick_count = state.tick_count.wrapping_add(1);
            if !state.error_persistent && state.error_timer > 0 {
                state.error_timer -= 1;
                if state.error_timer == 0 {
                    state.error = None;
                    state.error_persistent = false;
                }
            }
            if state.explorer.transfer_message.is_some()
                && state.tick_count >= state.explorer.transfer_message_clear_tick
            {
                state.explorer.transfer_message = None;
            }
            if state.explorer.transfer_error.is_some()
                && state.tick_count >= state.explorer.transfer_error_clear_tick
            {
                state.explorer.transfer_error = None;
            }
        }

        AppEvent::CheckUpdate => {
            if let Some((version, url)) = state.update_available.take() {
                commands.push(Command::DownloadUpdate { version, download_url: url });
                state.error = Some("Downloading update...".to_string());
                state.error_timer = 5;
            } else {
                commands.push(Command::CheckUpdate);
                state.error = Some("Checking for updates...".to_string());
                state.error_timer = 5;
            }
        }
        AppEvent::UpdateAvailable(version, url) => {
            state.update_available = Some((version.clone(), url.clone()));
        }
        AppEvent::Error(msg) => {
            state.error = Some(msg.clone());
            state.error_persistent = true;
        }
        AppEvent::Info(msg) => {
            if msg.is_empty() {
                state.error = None;
                state.error_persistent = false;
                state.error_timer = 0;
            } else {
                state.error = Some(msg.clone());
                state.error_timer = 5;
                state.error_persistent = false;
            }
        }

        AppEvent::DockerReconnecting => {
            state.containers.docker_reconnecting = true;
            state.containers.loading = true;
        }
        AppEvent::DockerReconnected => {
            state.containers.docker_reconnecting = false;
            state.containers.docker_connected = true;
            state.containers.loading = false;
        }
        AppEvent::DockerConnectionLost(reason) => {
            state.containers.docker_connected = false;
            state.containers.docker_reconnecting = false;
            state.containers.loading = false;
            state.error = Some(reason.clone());
            state.error_timer = 10;
        }

        // Delegate to sub-reducers
        _ => {
            use crate::app::event::AppEvent;
            match &event {
                AppEvent::Navigate(_)
                | AppEvent::Back
                | AppEvent::ShowHelp
                | AppEvent::HideHelp
                | AppEvent::ScrollHelp(_)
                | AppEvent::ShowConfirmDialog(_, _)
                | AppEvent::ConfirmYes
                | AppEvent::ConfirmNo
                | AppEvent::ShowDetails
                | AppEvent::Inspected(_, _)
                | AppEvent::ScrollDetails(_)
                | AppEvent::FilterSubmit(_)
                | AppEvent::JumpTop
                | AppEvent::JumpBottom => {
                    commands.extend(crate::app::reducers::navigation::reduce(state, &event));
                }
                AppEvent::ContainersUpdated(_)
                | AppEvent::SelectContainer(_)
                | AppEvent::FilterContainers(_)
                | AppEvent::ActivateFilter
                | AppEvent::RestartContainer(_)
                | AppEvent::StopContainer(_)
                | AppEvent::ContainerStopped(_)
                | AppEvent::ContainerStarted(_)
                | AppEvent::StartContainer(_)
                | AppEvent::ContainerDeleted(_)
                | AppEvent::ToggleSelectionMode
                | AppEvent::ToggleSelectContainer(_)
                | AppEvent::SelectAllContainers
                | AppEvent::ToggleColumnPicker
                | AppEvent::ToggleColumn(_)
                | AppEvent::BatchToggleContainers(_)
                | AppEvent::CycleStatusFilter => {
                    commands.extend(crate::app::reducers::container::reduce(state, &event));
                }
                AppEvent::ImagesUpdated(_)
                | AppEvent::SelectImage(_)
                | AppEvent::FilterImages(_)
                | AppEvent::ActivateImageFilter
                | AppEvent::PrunedImages(_)
                | AppEvent::RunImage(_, _)
                | AppEvent::ImageRunFieldUpdate(_, _)
                | AppEvent::ImageRunToggleAutoremove
                | AppEvent::ImageRunToggleAdvanced
                | AppEvent::ImageRunFocusNext
                | AppEvent::ImageRunFocusPrev
                | AppEvent::ImageRunSubmit => {
                    commands.extend(crate::app::reducers::image::reduce(state, &event));
                }
                AppEvent::LogLines(_, _)
                | AppEvent::TogglePause
                | AppEvent::ActivateLogSearch
                | AppEvent::SearchLogs(_)
                | AppEvent::SubmitLogSearch
                | AppEvent::ScrollLogs(_)
                | AppEvent::ToggleLogTimestamps
                | AppEvent::ExportLogs(_) => {
                    commands.extend(crate::app::reducers::log::reduce(state, &event));
                }
                AppEvent::EventsUpdated(_)
                | AppEvent::ActivateEventsFilter
                | AppEvent::EventsFilterSubmit
                | AppEvent::FilterEvents(_)
                | AppEvent::ScrollEvents(_) => {
                    commands.extend(crate::app::reducers::event::reduce(state, &event));
                }
                AppEvent::StatisticsUpdated(_)
                | AppEvent::CycleSortStat(_)
                | AppEvent::ToggleSortDirection => {
                    commands.extend(crate::app::reducers::statistics::reduce(state, &event));
                }
                AppEvent::NetworksUpdated(_)
                | AppEvent::SelectNetwork(_) => {
                    commands.extend(crate::app::reducers::network::reduce(state, &event));
                }
                AppEvent::VolumesUpdated(_)
                | AppEvent::SelectVolume(_) => {
                    commands.extend(crate::app::reducers::volume::reduce(state, &event));
                }
                AppEvent::CloseShell
                | AppEvent::StartShell(_, _, _, _)
                | AppEvent::ShellConfigSubmit
                | AppEvent::ShellConfigFieldUpdate(_, _)
                | AppEvent::ShellConfigFocusNext
                | AppEvent::ShellConfigFocusPrev => {
                    commands.extend(crate::app::reducers::shell::reduce(state, &event));
                }
                AppEvent::ExplorerSelect
                | AppEvent::ExplorerCopyToContainer
                | AppEvent::ExplorerCopyFromContainer
                | AppEvent::ExplorerTransferComplete(_)
                | AppEvent::ExplorerTransferError(_)
                | AppEvent::ExplorerFilter(_)
                | AppEvent::ExplorerContainerDirUpdated(_, _, _)
                | AppEvent::ExplorerHostGoUp
                | AppEvent::ExplorerContainerGoUp
                | AppEvent::ExplorerHostSelect(_)
                | AppEvent::ExplorerContainerSelect(_)
                | AppEvent::ExplorerHostEnterDir(_)
                | AppEvent::ExplorerContainerEnterDir(_)
                | AppEvent::ExplorerHostRefresh
                | AppEvent::ExplorerContainerRefresh
                | AppEvent::ExplorerHostActivateFilter
                | AppEvent::ExplorerContainerActivateFilter
                | AppEvent::ExplorerFilterSubmit
                | AppEvent::ExplorerHostActivateRename
                | AppEvent::ExplorerContainerActivateRename
                | AppEvent::ExplorerRenameUpdate(_)
                | AppEvent::ExplorerRenameSubmit
                | AppEvent::ExplorerHostDirUpdated(_, _) => {
                    commands.extend(crate::app::reducers::explorer::reduce(state, &event));
                }
                _ => {}
            }
        }
    }

    commands
}
