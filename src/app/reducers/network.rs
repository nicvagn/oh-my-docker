use crate::app::event::{AppEvent, Command};
use crate::app::state::AppState;

pub fn reduce(state: &mut AppState, event: &AppEvent) -> Vec<Command> {
    match event {
        AppEvent::NetworksUpdated(networks) => {
            state.networks.items = networks.clone();
            state.networks.loading = false;
        }
        AppEvent::SelectNetwork(idx) if *idx < state.networks.items.len() => {
            state.networks.selected = *idx;
        }
        _ => {}
    }
    Vec::new()
}
