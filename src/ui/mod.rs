use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;
use crate::app::mode;
use crate::app::mode::Mode;
use crate::app::state::AppState;

pub fn staleness_indicator(last_updated: Option<std::time::Instant>, fresh: Duration, stale: Duration) -> (char, Color) {
    match last_updated {
        Some(instant) => {
            let elapsed = instant.elapsed();
            if elapsed < fresh { ('●', Color::Green) }
            else if elapsed < stale { ('○', Color::Yellow) }
            else { ('◌', Color::Red) }
        }
        None => ('?', Color::DarkGray),
    }
}

pub mod column_picker;
pub mod containers;
pub mod container_details;
pub mod logs;
pub mod images;
pub mod shell;
pub mod shell_config;
pub mod events;
pub mod statistics;
pub mod networks;
pub mod volumes;
pub mod help;
pub mod confirm_dialog;
pub mod tabs_bar;
pub mod status_bar;
pub mod theme;

pub fn render(frame: &mut Frame, state: &mut AppState) {
    let current = state.navigation.mode_stack.current();
    let is_base = mode::mode_to_tab(current).is_some();
    let area = frame.area();

    if is_base {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // header
                Constraint::Min(0),     // content
                Constraint::Length(1),  // status bar
            ])
            .split(area);

        tabs_bar::render(frame, chunks[0], state.selected_tab);
        status_bar::render(frame, chunks[2], state.selected_tab);
        render_content(frame, state, chunks[1]);
    } else {
        render_content(frame, state, area);
    }

    if let Some(ref update) = state.update_available {
        let msg = format!("Update v{} available — press U to download", update.0);
        render_update_toast(frame, &msg);
    }

    if let Some(err) = &state.error {
        render_error_toast(frame, err, state.error_persistent);
    }
}

fn render_content(frame: &mut Frame, state: &mut AppState, area: Rect) {
    match state.navigation.mode_stack.current() {
        Mode::Containers => containers::render(frame, area, &state.containers, state.tick_count, &state.config.container_columns),
        Mode::ContainerDetails(_) => {
            if let Some(ref mut details) = state.navigation.details {
                container_details::render(frame, area, details, &state.containers);
            } else {
                container_details::render_placeholder(frame, area);
            }
        }
        Mode::Logs(_) => {
            if let Some(ref mut logs) = state.navigation.logs {
                logs::render(frame, area, logs);
            } else {
                logs_render_placeholder(frame, area);
            }
        }
        Mode::Images => images::render(frame, area, &state.images, &state.config.image_columns),
        Mode::ImageRun(_) => {
            if let Some(ref run) = state.navigation.image_run {
                images::render_run(frame, area, run);
            }
        }
        Mode::Shell(_) => {
            if let Some(ref shell) = state.navigation.shell {
                shell::render(frame, area, shell);
            } else {
                shell_render_placeholder(frame, area);
            }
        }
        Mode::ShellConfig(_) => {
            if let Some(ref cfg) = state.navigation.shell_config {
                shell_config::render(frame, area, cfg);
            }
        }
        Mode::Events => events::render(frame, area, &mut state.events),
        Mode::Statistics => statistics::render(frame, area, &state.statistics),
        Mode::Networks => networks::render(frame, area, &state.networks, &state.config.network_columns),
        Mode::Volumes => volumes::render(frame, area, &state.volumes, &state.config.volume_columns),
        Mode::Help => help::render(frame, area, &mut state.navigation.help, &state.config),
        Mode::ConfirmDialog { .. } => confirm_dialog::render(frame, area, state.navigation.mode_stack.current()),
    }
}

fn logs_render_placeholder(frame: &mut Frame, area: Rect) {
    use ratatui::layout::Alignment;
    use ratatui::text::Text;
    use ratatui::widgets::{Block, Borders, BorderType};
    let block = Block::default()
        .title(" LOGS ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Green));
    let text = Text::from(vec![
        "  No log session active".into(),
        "".into(),
        "  Esc  back".into(),
    ]);
    frame.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::White)).block(block),
        area,
    );
}

fn shell_render_placeholder(frame: &mut Frame, area: Rect) {
    use ratatui::layout::Alignment;
    use ratatui::text::Text;
    use ratatui::widgets::{Block, Borders, BorderType};
    let block = Block::default()
        .title(" SHELL ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Green));
    let text = Text::from(vec![
        "  No active shell session".into(),
        "".into(),
        "  Esc  back".into(),
    ]);
    frame.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::White)).block(block),
        area,
    );
}

fn render_filter_bar(frame: &mut Frame, area: Rect, filter: &str, placeholder: &str) {
    use ratatui::widgets::Paragraph;
    let filter_area = Rect {
        x: area.x + 1,
        y: area.y,
        width: area.width.saturating_sub(2).min(40),
        height: 1,
    };
    let display = if filter.is_empty() {
        format!("/  {}...", placeholder)
    } else {
        filter.to_string()
    };
    frame.render_widget(
        Paragraph::new(format!("/{}", display))
            .style(Style::default().fg(Color::White).bg(Color::DarkGray)),
        filter_area,
    );
}

fn render_update_toast(frame: &mut Frame, message: &str) {
    let area = frame.area();
    let toast_area = ratatui::layout::Rect {
        x: 1,
        y: area.height.saturating_sub(2),
        width: (message.len() as u16 + 4).min(area.width),
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(format!(" {} ", message))
            .style(Style::default().fg(Color::Black).bg(Color::Yellow)),
        toast_area,
    );
}

fn render_error_toast(frame: &mut Frame, message: &str, persistent: bool) {
    let area = frame.area();
    let suffix = if persistent { "  [any key]" } else { "" };
    let full = format!(" {}{} ", message, suffix);
    let toast_area = ratatui::layout::Rect {
        x: area.width.saturating_sub(full.len() as u16 + 2).min(1),
        y: area.height.saturating_sub(3),
        width: (full.len() as u16 + 2).min(area.width),
        height: 1,
    };
    let bg = if persistent { Color::Red } else { Color::DarkGray };
    frame.render_widget(
        Paragraph::new(full)
            .style(Style::default().fg(Color::White).bg(bg)),
        toast_area,
    );
}
