use std::time::Instant;

use crate::app::event::{AppEvent, Command};
use crate::app::state::AppState;

pub fn reduce(state: &mut AppState, event: &AppEvent) -> Vec<Command> {
    let mut commands = Vec::new();
    match event {
        AppEvent::VolumesUpdated(volumes) => {
            state.volumes.items = volumes.clone();
            state.volumes.loading = false;
            state.volumes.last_updated = Some(Instant::now());
        }
        AppEvent::SelectVolume(idx) if *idx < state.volumes.items.len() => {
            state.volumes.selected = *idx;
        }
        AppEvent::ToggleColumnPicker => {
            state.volumes.show_column_picker = !state.volumes.show_column_picker;
        }
        AppEvent::ToggleColumn(name) => {
            if state.volumes.show_column_picker {
                let col_count = 3;
                crate::app::reducers::handle_column_nav(name, col_count, &mut state.volumes.column_picker_selection);
                match name.as_str() {
                    "name" => state.config.volume_columns.show_name = !state.config.volume_columns.show_name,
                    "driver" => state.config.volume_columns.show_driver = !state.config.volume_columns.show_driver,
                    "mountpoint" => state.config.volume_columns.show_mountpoint = !state.config.volume_columns.show_mountpoint,
                    _ => {}
                }
                commands.push(Command::SaveConfig);
            }
        }
        _ => {}
    }
    commands
}
