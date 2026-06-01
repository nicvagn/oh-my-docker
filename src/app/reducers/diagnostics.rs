use crate::app::event::{AppEvent, Command};
use crate::app::mode::Mode;
use crate::app::state::{AppState, DiagnosticsPhase};
use crate::config::OmdockerConfig;

pub fn reduce(state: &mut AppState, event: &AppEvent) -> Vec<Command> {
    let mut commands = Vec::new();
    match event {
        AppEvent::StartDiagnostics(container_id) => {
            // Reload config from disk so user edits take effect immediately
            let (config, parse_err) = OmdockerConfig::load();
            state.config = config;
            state.rebuild_keymap();

            if let Some(ref e) = parse_err {
                let msg = format!("Config file has errors — cannot run diagnostics.\n\n{}", e);
                state.navigation.mode_stack.push(Mode::InfoDialog(msg));
                return commands;
            }

            if state.config.llm.is_none() {
                let msg = concat!(
                    "AI diagnostics not configured.\n\n",
                    "Add an [llm] section to:\n",
                    "  ~/.config/omdocker/omdocker.toml\n\n",
                    "Example:\n",
                    "  [llm]\n",
                    "  provider = \"ollama\"\n",
                    "  model = \"llama3\"\n",
                    "  # base_url = \"http://localhost:11434\"\n\n",
                    "Supported providers: ollama, openai, anthropic"
                );
                state.navigation.mode_stack.push(Mode::InfoDialog(msg.to_string()));
                return commands;
            }

            state.navigation.diagnostics = Some(crate::app::state::DiagnosticsState::new(container_id.clone()));
            let current = state.navigation.mode_stack.current().clone();
            if !matches!(current, Mode::Diagnostics(_)) {
                state.navigation.mode_stack.push(Mode::Diagnostics(container_id.clone()));
            }
            commands.push(Command::StartDiagnostics(container_id.clone()));
        }
        AppEvent::DiagnosticsPhaseUpdate(phase) => {
            if let Some(ref mut d) = state.navigation.diagnostics {
                d.phase = phase.clone();
            }
        }
        AppEvent::DiagnosticsChunk(chunk) => {
            if let Some(ref mut d) = state.navigation.diagnostics {
                d.analysis.push_str(chunk);
                d.analysis.push(' ');
            }
        }
        AppEvent::DiagnosticsPlaybook(chunk) => {
            if let Some(ref mut d) = state.navigation.diagnostics {
                d.playbook.push_str(chunk);
                d.playbook.push(' ');
            }
        }
        AppEvent::DiagnosticsDone => {
            if let Some(ref mut d) = state.navigation.diagnostics {
                d.phase = DiagnosticsPhase::Done;
            }
        }
        AppEvent::DiagnosticsError(msg) => {
            if let Some(ref mut d) = state.navigation.diagnostics {
                d.phase = DiagnosticsPhase::Error(msg.clone());
            }
        }
        AppEvent::ScrollDiagnostics(delta) => {
            if let Some(ref mut d) = state.navigation.diagnostics {
                if *delta < 0 {
                    d.scroll_offset = d.scroll_offset.saturating_sub((-delta) as usize);
                } else {
                    d.scroll_offset = d.scroll_offset.saturating_add(*delta as usize);
                }
            }
        }
        _ => {}
    }
    commands
}
