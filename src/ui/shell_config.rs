use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, BorderType};

use crate::app::state::ShellConfigState;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, config: &ShellConfigState) {
    let block = Block::default()
        .title(format!(" SHELL CONFIG — {} ", &config.container_id[..12.min(config.container_id.len())]))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::view_border()));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    // --- Field: Shell ---
    let shell_val = if config.shell.is_empty() { "bash".to_string() } else { config.shell.clone() };
    lines.push(Line::from(Span::styled(
        format!(" {} Shell", if config.field_focus == 0 { "▸" } else { " " }),
        Style::default().fg(if config.field_focus == 0 { Color::White } else { Color::DarkGray }),
    )));
    lines.push(Line::from(Span::styled(
        format!("    {}", shell_val),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));

    // --- Field: User ---
    let user_display = if config.user.is_empty() {
        "(container default)".to_string()
    } else {
        config.user.clone()
    };
    lines.push(Line::from(Span::styled(
        format!(" {} User", if config.field_focus == 1 { "▸" } else { " " }),
        Style::default().fg(if config.field_focus == 1 { Color::White } else { Color::DarkGray }),
    )));
    lines.push(Line::from(Span::styled(
        format!("    {}", user_display),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));

    // --- Field: Workdir ---
    let wd_display = if config.workdir.is_empty() {
        "(container default)".to_string()
    } else {
        config.workdir.clone()
    };
    lines.push(Line::from(Span::styled(
        format!(" {} Workdir", if config.field_focus == 2 { "▸" } else { " " }),
        Style::default().fg(if config.field_focus == 2 { Color::White } else { Color::DarkGray }),
    )));
    lines.push(Line::from(Span::styled(
        format!("    {}", wd_display),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));

    // Help for current field
    lines.push(Line::from(""));
    let help = match config.field_focus {
        0 => "Valid values: sh, bash, or a custom path like /bin/zsh",
        1 => "Empty = container default | host = current host user | root | user:group",
        _ => "Empty = container default | / = root | or a custom path like /app",
    };
    let val_style =    if config.field_focus == 0 && config.shell.is_empty() {
        Some("bash")
    } else if (config.field_focus == 1 && config.user.is_empty()) || (config.field_focus == 2 && config.workdir.is_empty()) {
        Some("(default)")
    } else {
        None
    };
    if let Some(default_label) = val_style {
        lines.push(Line::from(Span::styled(
            format!("  {}  (type or press Enter for {})", help, default_label),
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            format!("  {}", help),
            Style::default().fg(Color::DarkGray),
        )));
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .style(Style::default().fg(Color::White))
        .block(Block::default());

    frame.render_widget(paragraph, Rect {
        x: inner.x + 1,
        y: inner.y,
        width: inner.width.saturating_sub(2),
        height: inner.height,
    });
}
