use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, BorderType, Clear, Paragraph};

use crate::app::mode::Mode;

pub fn render(frame: &mut Frame, mode: &Mode) {
    let Mode::ConfirmDialog { prompt, .. } = mode else {
        return;
    };

    let area = frame.area();

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  ⚠  {}", prompt),
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  y/Enter: confirm    n/Esc: cancel",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
    ];

    let text = Text::from(lines);

    let width = (prompt.len() as u16 + 12).min(area.width).max(40);
    let height = 7u16;

    let dialog_area = Rect {
        x: area.x + (area.width.saturating_sub(width)) / 2,
        y: area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    };

    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" CONFIRM ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::White))
        .block(block);

    frame.render_widget(paragraph, dialog_area);
}
