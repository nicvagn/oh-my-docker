use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use crate::app::mode;
use crate::app::mode::Mode;
use crate::app::state::AppState;

pub const SPINNER_CHARS: [char; 9] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠏'];

pub fn spinner_char(tick_count: u64) -> char {
    SPINNER_CHARS[(tick_count as usize / 2) % SPINNER_CHARS.len()]
}

pub fn staleness_indicator(last_updated: Option<std::time::Instant>, interval_ms: u64) -> (char, Color) {
    let fresh = Duration::from_millis(interval_ms * 2);
    let stale = Duration::from_millis(interval_ms * 5);
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
pub mod container_details;
pub mod diagnostics;
pub mod explorer;
pub mod logs;
pub mod resource_panel;
pub mod shell;
pub mod shell_config;
pub mod events;
pub mod statistics;
pub mod help;
pub mod image_run;
pub mod confirm_dialog;
pub mod tabs_bar;
pub mod status_bar;
pub mod theme;

pub fn render(frame: &mut Frame, state: &mut AppState) {
    let area = frame.area();
    state.term_width = area.width;
    let current = state.navigation.mode_stack.current();
    let is_base = mode::mode_to_tab(current).is_some();

    if is_base {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // header
                Constraint::Min(0),     // content
                Constraint::Length(1),  // status bar
            ])
            .split(area);

        tabs_bar::render(frame, chunks[0], state.selected_tab, &state.containers.items);
        status_bar::render(frame, chunks[2], state.navigation.mode_stack.current());
        render_content(frame, state, chunks[1]);
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(area);
        status_bar::render(frame, chunks[1], state.navigation.mode_stack.current());
        render_content(frame, state, chunks[0]);
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
        Mode::Containers => resource_panel::render_containers(frame, area, &mut state.containers, &state.container_extra, state.tick_count, state.config.polling.containers_ms),
        Mode::ContainerDetails(_) => {
            if let Some(ref mut details) = state.navigation.details {
                container_details::render(frame, area, details, &state.containers, &state.container_extra);
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
        Mode::Images => resource_panel::render_simple_list::<resource_panel::ImageResource>(frame, area, &mut state.images, state.tick_count, state.config.polling.images_ms),
        Mode::ImageRun(_) => {
            if let Some(ref run) = state.navigation.image_run {
                crate::ui::image_run::render_run(frame, area, run);
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
        Mode::Statistics => statistics::render(frame, area, &state.statistics, state.tick_count),
        Mode::Networks => resource_panel::render_simple_list::<resource_panel::NetworkResource>(frame, area, &mut state.networks, state.tick_count, state.config.polling.networks_ms),
        Mode::Volumes => resource_panel::render_simple_list::<resource_panel::VolumeResource>(frame, area, &mut state.volumes, state.tick_count, state.config.polling.volumes_ms),
        Mode::Explorer(_) | Mode::ExplorerVolume(_, _) => explorer::render(frame, area, state),
        Mode::Help => help::render(frame, area, &mut state.navigation.help, &state.config),
        Mode::ConfirmDialog { .. } => confirm_dialog::render(frame, area, state.navigation.mode_stack.current()),
        Mode::InfoDialog(_) => render_info_dialog(frame, area, state.navigation.mode_stack.current()),
        Mode::Diagnostics(_) => {
            if let Some(ref mut d) = state.navigation.diagnostics {
                diagnostics::render(frame, area, d);
            }
        }
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

fn render_info_dialog(frame: &mut Frame, area: Rect, mode: &Mode) {
    let Mode::InfoDialog(message) = mode else {
        return;
    };

    let max_line_width = message.lines().map(|l| l.len()).max().unwrap_or(40) as u16;
    let width = (max_line_width + 8).min(area.width).max(44);
    let line_count = message.lines().count() as u16;
    let height = (line_count + 5).min(area.height).max(8);

    let dialog_area = Rect {
        x: area.x + (area.width.saturating_sub(width)) / 2,
        y: area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    };

    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" AI DIAGNOSTICS ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow));

    let mut full_lines: Vec<Line> = vec![Line::from("")];
    for l in message.lines() {
        full_lines.push(Line::from(Span::styled(
            format!("  {}", l),
            Style::default().fg(Color::White),
        )));
    }
    full_lines.push(Line::from(""));
    full_lines.push(Line::from(Span::styled(
        "  Press any key to close",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(Text::from(full_lines))
        .style(Style::default().fg(Color::White))
        .block(block);

    frame.render_widget(paragraph, dialog_area);
}
