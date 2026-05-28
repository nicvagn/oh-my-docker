use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq)]
pub enum Mode {
    Containers,
    ContainerDetails(String),
    Logs(String),
    Images,
    ImageRun(String),
    Shell(String),
    ShellConfig(String),
    Events,
    Statistics,
    Networks,
    Volumes,
    Explorer(String),
    Help,
    ConfirmDialog {
        prompt: String,
        action: crate::app::event::ConfirmAction,
    },
}

#[derive(Clone, Debug)]
pub struct ModeStack {
    stack: VecDeque<Mode>,
    max_depth: usize,
}

impl ModeStack {
    pub fn new() -> Self {
        Self {
            stack: VecDeque::from([Mode::Containers]),
            max_depth: 10,
        }
    }

    pub fn current(&self) -> &Mode {
        self.stack.back().unwrap_or(&Mode::Containers)
    }

    pub fn push(&mut self, mode: Mode) {
        if self.stack.len() >= self.max_depth {
            self.stack.pop_front();
        }
        self.stack.push_back(mode);
    }

    pub fn back(&mut self) -> Option<Mode> {
        if self.stack.len() > 1 {
            self.stack.pop_back()
        } else {
            None
        }
    }

    pub fn replace_current(&mut self, mode: Mode) {
        self.stack.pop_back();
        self.stack.push_back(mode);
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }
}

impl Default for ModeStack {
    fn default() -> Self {
        Self::new()
    }
}

pub fn mode_to_tab(mode: &Mode) -> Option<usize> {
    match mode {
        Mode::Containers => Some(0),
        Mode::Images => Some(1),
        Mode::Networks => Some(2),
        Mode::Volumes => Some(3),
        Mode::Events => Some(4),
        Mode::Statistics => Some(5),
        Mode::Help => Some(6),
        _ => None,
    }
}

pub fn tab_to_mode(tab: usize) -> Mode {
    match tab {
        0 => Mode::Containers,
        1 => Mode::Images,
        2 => Mode::Networks,
        3 => Mode::Volumes,
        4 => Mode::Events,
        5 => Mode::Statistics,
        6 => Mode::Help,
        _ => Mode::Containers,
    }
}

pub const TAB_TITLES: [&str; 7] = ["Containers", "Images", "Networks", "Volumes", "Events", "Statistics", "Help"];

pub const TAB_COUNT: usize = 7;
