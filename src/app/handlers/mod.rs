pub mod container;
pub mod event;
pub mod image;
pub mod log;
pub mod navigation;
pub mod network;
pub mod shell;
pub mod statistics;
pub mod volume;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug)]
pub enum FilterAction {
    Update(String),
    Clear,
    Submit(Option<usize>),
}

pub fn handle_filter_input(
    key: KeyEvent,
    current_filter: &str,
    current_selection: usize,
    max_selection: usize,
) -> Option<FilterAction> {
    match (key.code, key.modifiers) {
        (KeyCode::Backspace, _) | (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
            let new_q = current_filter.chars().take(current_filter.chars().count().saturating_sub(1)).collect();
            Some(FilterAction::Update(new_q))
        }
        (KeyCode::Esc, _) => Some(FilterAction::Clear),
        (KeyCode::Enter, _) => Some(FilterAction::Submit(None)),
        (KeyCode::Down, _) => {
            let next = (current_selection + 1).min(max_selection.saturating_sub(1));
            Some(FilterAction::Submit(Some(next)))
        }
        (KeyCode::Up, _) => {
            let prev = current_selection.saturating_sub(1);
            Some(FilterAction::Submit(Some(prev)))
        }
        (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
            Some(FilterAction::Update(format!("{}{}", current_filter, c)))
        }
        _ => None,
    }
}
