use crate::app::mode::ModeStack;
use crate::app::state::{DetailsState, DiagnosticsState, HelpState, ImageRunState, LogState, ShellConfigState, ShellState};

#[derive(Clone, Debug)]
pub struct NavigationState {
    pub mode_stack: ModeStack,
    pub details: Option<DetailsState>,
    pub logs: Option<LogState>,
    pub image_run: Option<ImageRunState>,
    pub shell: Option<ShellState>,
    pub shell_config: Option<ShellConfigState>,
    pub help: HelpState,
    pub diagnostics: Option<DiagnosticsState>,
}

impl NavigationState {
    pub fn new() -> Self {
        Self {
            mode_stack: ModeStack::new(),
            details: None,
            logs: None,
            image_run: None,
            shell: None,
            shell_config: None,
            help: HelpState::default(),
            diagnostics: None,
        }
    }
}
