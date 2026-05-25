#[derive(Clone, Debug)]
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
    Help,
    ConfirmDialog {
        prompt: String,
        action: crate::app::event::ConfirmAction,
    },
}

impl PartialEq for Mode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Mode::Containers, Mode::Containers) => true,
            (Mode::ContainerDetails(a), Mode::ContainerDetails(b)) => a == b,
            (Mode::Logs(a), Mode::Logs(b)) => a == b,
            (Mode::Images, Mode::Images) => true,
            (Mode::ImageRun(a), Mode::ImageRun(b)) => a == b,
            (Mode::Shell(a), Mode::Shell(b)) => a == b,
            (Mode::ShellConfig(a), Mode::ShellConfig(b)) => a == b,
            (Mode::Events, Mode::Events) => true,
            (Mode::Statistics, Mode::Statistics) => true,
            (Mode::Networks, Mode::Networks) => true,
            (Mode::Volumes, Mode::Volumes) => true,
            (Mode::Help, Mode::Help) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ModeStack {
    stack: Vec<Mode>,
    max_depth: usize,
}

impl ModeStack {
    pub fn new() -> Self {
        Self {
            stack: vec![Mode::Containers],
            max_depth: 10,
        }
    }

    pub fn current(&self) -> &Mode {
        self.stack.last().unwrap_or(&Mode::Containers)
    }

    pub fn push(&mut self, mode: Mode) {
        if self.stack.len() >= self.max_depth {
            self.stack.remove(0);
        }
        self.stack.push(mode);
    }

    pub fn back(&mut self) -> Option<Mode> {
        if self.stack.len() > 1 {
            Some(self.stack.pop().unwrap())
        } else {
            None
        }
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
