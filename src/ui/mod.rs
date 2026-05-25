use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;
use crate::app::mode::Mode;
use crate::app::state::AppState;

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

pub fn render(frame: &mut Frame, state: &mut AppState) {
    match state.mode_stack.current() {
        Mode::Containers => containers::render(frame, &state.containers),
        Mode::ContainerDetails(_) => {
            if let Some(ref mut details) = state.details {
                container_details::render(frame, details);
            } else {
                container_details::render_placeholder(frame);
            }
        }
        Mode::Logs(_) => {
            if let Some(ref logs) = state.logs {
                logs::render(frame, logs);
            } else {
                logs_render_placeholder(frame);
            }
        }
        Mode::Images => images::render(frame, &state.images),
        Mode::ImageRun(_) => {
            if let Some(ref run) = state.image_run {
                images::render_run(frame, run);
            }
        }
        Mode::Shell(_) => {
            if let Some(ref shell) = state.shell {
                shell::render(frame, shell);
            } else {
                shell_render_placeholder(frame);
            }
        }
        Mode::ShellConfig(_) => {
            if let Some(ref cfg) = state.shell_config {
                shell_config::render(frame, cfg);
            }
        }
        Mode::Events => events::render(frame, &state.events),
        Mode::Statistics => statistics::render(frame, &state.statistics),
        Mode::Networks => networks::render(frame, &state.networks),
        Mode::Volumes => volumes::render(frame, &state.volumes),
        Mode::Help => help::render(frame, &mut state.help),
        Mode::ConfirmDialog { .. } => confirm_dialog::render(frame, state.mode_stack.current()),
    }

    if let Some(ref update) = state.update_available {
        let msg = format!("Update v{} available — press U to download", update.0);
        render_update_toast(frame, &msg);
    }

    if let Some(err) = &state.error {
        render_error_toast(frame, err);
    }
}

fn logs_render_placeholder(frame: &mut Frame) {
    use ratatui::layout::Alignment;
    use ratatui::text::Text;
    use ratatui::widgets::{Block, Borders, BorderType};
    let block = Block::default()
        .title(" LOGS ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));
    let text = Text::from(vec![
        "  No log session active".into(),
        "".into(),
        "  Esc  back".into(),
    ]);
    frame.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::White)).block(block),
        frame.area(),
    );
}

fn shell_render_placeholder(frame: &mut Frame) {
    use ratatui::layout::Alignment;
    use ratatui::text::Text;
    use ratatui::widgets::{Block, Borders, BorderType};
    let block = Block::default()
        .title(" SHELL ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));
    let text = Text::from(vec![
        "  No active shell session".into(),
        "".into(),
        "  Esc  back".into(),
    ]);
    frame.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::White)).block(block),
        frame.area(),
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

fn render_error_toast(frame: &mut Frame, message: &str) {
    let area = frame.area();
    let toast_area = ratatui::layout::Rect {
        x: area.width.saturating_sub(message.len() as u16 + 4).min(1),
        y: area.height.saturating_sub(3),
        width: (message.len() as u16 + 4).min(area.width),
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(format!(" {} ", message))
            .style(Style::default().fg(Color::White).bg(Color::Red)),
        toast_area,
    );
}
