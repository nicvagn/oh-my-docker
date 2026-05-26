use crate::app::event::{AppEvent, Command};
use crate::app::state::AppState;

pub fn reduce(state: AppState, event: AppEvent) -> (AppState, Vec<Command>) {
    let mut new_state = state;
    let mut commands = Vec::new();

    match &event {
        // Global events handled inline
        AppEvent::Quit => new_state.quit = true,

        AppEvent::Tick => {
            new_state.tick_count = new_state.tick_count.wrapping_add(1);
            if !new_state.error_persistent && new_state.error_timer > 0 {
                new_state.error_timer -= 1;
                if new_state.error_timer == 0 {
                    new_state.error = None;
                    new_state.error_persistent = false;
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
            new_state.update_available = Some((version.clone(), url.clone()));
        }
        AppEvent::Error(msg) => {
            new_state.error = Some(msg.clone());
            new_state.error_persistent = true;
        }
        AppEvent::Info(msg) => {
            if msg.is_empty() {
                new_state.error = None;
                new_state.error_persistent = false;
                new_state.error_timer = 0;
            } else {
                new_state.error = Some(msg.clone());
                new_state.error_timer = 5;
                new_state.error_persistent = false;
            }
        }

        AppEvent::DockerReconnecting => {
            new_state.containers.docker_reconnecting = true;
            new_state.containers.loading = true;
        }
        AppEvent::DockerReconnected => {
            new_state.containers.docker_reconnecting = false;
            new_state.containers.docker_connected = true;
            new_state.containers.loading = false;
        }
        AppEvent::DockerConnectionLost(reason) => {
            new_state.containers.docker_connected = false;
            new_state.containers.docker_reconnecting = false;
            new_state.containers.loading = false;
            new_state.error = Some(reason.clone());
            new_state.error_timer = 10;
        }

        // Delegate to sub-reducers
        _ => {
            commands.extend(crate::app::reducers::navigation::reduce(&mut new_state, &event));
            commands.extend(crate::app::reducers::container::reduce(&mut new_state, &event));
            commands.extend(crate::app::reducers::image::reduce(&mut new_state, &event));
            commands.extend(crate::app::reducers::log::reduce(&mut new_state, &event));
            commands.extend(crate::app::reducers::event::reduce(&mut new_state, &event));
            commands.extend(crate::app::reducers::statistics::reduce(&mut new_state, &event));
            commands.extend(crate::app::reducers::network::reduce(&mut new_state, &event));
            commands.extend(crate::app::reducers::volume::reduce(&mut new_state, &event));
            commands.extend(crate::app::reducers::shell::reduce(&mut new_state, &event));
        }
    }

    (new_state, commands)
}
