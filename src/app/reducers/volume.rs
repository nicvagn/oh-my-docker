use crate::app::event::{AppEvent, Command};
use crate::app::state::AppState;

pub fn reduce(state: &mut AppState, event: &AppEvent) -> Vec<Command> {
    match event {
        AppEvent::VolumesUpdated(volumes) => {
            state.volumes.items = volumes.clone();
            state.volumes.loading = false;
        }
        AppEvent::SelectVolume(idx) if *idx < state.volumes.items.len() => {
            state.volumes.selected = *idx;
        }
        _ => {}
    }
    Vec::new()
}
