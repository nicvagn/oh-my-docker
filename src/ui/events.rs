use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, BorderType, Wrap};

use crate::app::state::EventsState;

pub fn render(frame: &mut Frame, state: &mut EventsState) {
    let search_label = if !state.filter.is_empty() {
        format!(" FILTER '{}'", state.filter)
    } else {
        String::new()
    };
    let title = format!(" DOCKER EVENTS{} ", search_label);

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(frame.area());

    if state.buffer.is_empty() {
        let text = Text::from(vec![
            Line::from(Span::styled("  Waiting for Docker events...", Style::default().fg(Color::Yellow))),
            Line::from(""),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), frame.area());
        render_bottom_bar(frame, inner);
        return;
    }

    let height = inner.height as usize;

    state.viewport_height = height;
    let max_offset = state.buffer.len().saturating_sub(height);
    state.scroll_offset = state.scroll_offset.min(max_offset);
    let start = state.buffer.len().saturating_sub(height + state.scroll_offset);

    let lines: Vec<Line> = state
        .buffer
        .iter()
        .skip(start)
        .take(height)
        .filter(|e| {
            state.filter.is_empty()
                || e.kind.contains(&state.filter)
                || e.action.contains(&state.filter)
                || e.actor.contains(&state.filter)
        })
        .map(|e| {
            let icon = match e.kind.as_str() {
                "container" => "\u{1f4e6}",
                "image" => "\u{1f5bc}",
                "network" => "\u{1f310}",
                "volume" => "\u{1f4be}",
                _ => "\u{2699}",
            };

            let color = match e.kind.as_str() {
                "container" => Color::Cyan,
                "image" => Color::Yellow,
                "network" => Color::Magenta,
                "volume" => Color::Green,
                _ => Color::White,
            };

            Line::from(Span::styled(
                format!("{} {}  {}  {}", icon, e.timestamp, e.kind, e.action),
                Style::default().fg(color),
            ))
        })
        .collect();

    // Pad with empty lines if fewer than viewport height to avoid leftover artifacts
    let mut text_lines = lines;
    while text_lines.len() < height {
        text_lines.push(Line::from(Span::styled("", Style::default())));
    }

    let text = Text::from(text_lines);
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, frame.area());

    render_bottom_bar(frame, inner);

    if state.filter_active {
        render_search_bar(frame, inner, &state.filter);
    }
}

fn render_bottom_bar(frame: &mut Frame, area: Rect) {
    let bar = Rect {
        x: area.x + 1,
        y: area.y + area.height.saturating_sub(1),
        width: area.width.saturating_sub(2),
        height: 1,
    };
    let text = if area.width >= 55 {
        "  / filter   e:export   ↑↓/k j line   PgUp/PgDn page   g/G top/bottom   Esc back"
    } else if area.width >= 40 {
        "  / filter   e:export   PgUp/PgDn page   Esc back"
    } else {
        "  / filter   e:export   Esc back"
    };
    frame.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::DarkGray)),
        bar,
    );
}

fn render_search_bar(frame: &mut Frame, area: Rect, search: &str) {
    let search_area = Rect {
        x: area.x + 1,
        y: area.y,
        width: area.width.saturating_sub(2).min(40),
        height: 1,
    };
    let display = if search.is_empty() {
        "/  filter..."
    } else {
        search
    };
    frame.render_widget(
        Paragraph::new(format!("/{}", display))
            .style(Style::default().fg(Color::White).bg(Color::DarkGray)),
        search_area,
    );
}