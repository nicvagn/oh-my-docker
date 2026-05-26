use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, BorderType};

use crate::app::state::LogState;

fn highlight_text(text: &str, search: &str) -> Line<'static> {
    let search_lower = search.to_lowercase();
    let text_lower = text.to_lowercase();
    let mut spans: Vec<Span> = Vec::new();
    let mut last_end: usize = 0;

    for (i, _) in text_lower.char_indices() {
        if text_lower[i..].starts_with(&search_lower) {
            if last_end < i {
                spans.push(Span::styled(
                    text[last_end..i].to_string(),
                    Style::default().fg(Color::Yellow),
                ));
            }

            spans.push(Span::styled(
                text[i..i + search.len()].to_string(),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));

            last_end = i + search.len();
        }
    }

    if last_end < text.len() {
        spans.push(Span::styled(
            text[last_end..].to_string(),
            Style::default().fg(Color::Yellow),
        ));
    }

    if spans.is_empty() {
        spans.push(Span::styled(
            text.to_string(),
            Style::default().fg(Color::Yellow).bg(Color::DarkGray),
        ));
    }

    Line::from(spans)
}

pub fn render(frame: &mut Frame, state: &mut LogState) {
    let search_label = if !state.search.is_empty() {
        format!(" SEARCH '{}'", state.search)
    } else {
        String::new()
    };
    let title = if state.paused {
        format!(" LOGS — {} (PAUSED{}) ", state.container_id, search_label)
    } else {
        format!(" LOGS — {} (LIVE{}) ", state.container_id, search_label)
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(frame.area());

    if state.buffer.is_empty() {
        let text = Text::from(vec![
            Line::from(Span::styled("  Waiting for logs...", Style::default().fg(Color::Yellow))),
            Line::from(""),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), frame.area());
        render_bottom_bar(frame, inner, state.paused);
        return;
    }

    let has_filter = !state.search.is_empty();
    let height = inner.height as usize;

    state.viewport_height = height;
    let max_offset = state.buffer.len().saturating_sub(height);
    state.scroll_offset = state.scroll_offset.min(max_offset);
    let start = state.buffer.len().saturating_sub(height + state.scroll_offset);
    let show_ts = state.show_timestamps;
    let lines: Vec<Line> = state
        .buffer
        .iter()
        .skip(start)
        .take(height)
        .map(|entry| {
            let message = entry.message.trim_end().to_string();

            if show_ts && !entry.timestamp.is_empty() {
                let display_text = format!("{} {}", entry.timestamp, message);
                if has_filter && display_text.to_lowercase().contains(&state.search.to_lowercase()) {
                    highlight_text(&display_text, &state.search)
                } else {
                    let ts = entry.timestamp.clone();
                    let msg = entry.message.trim_end().to_string();
                    Line::from(vec![
                        Span::styled(ts, Style::default().fg(Color::DarkGray)),
                        Span::styled(msg, Style::default().fg(Color::White)),
                    ])
                }
            } else if has_filter && message.to_lowercase().contains(&state.search.to_lowercase()) {
                highlight_text(&message, &state.search)
            } else {
                Line::from(Span::styled(message, Style::default().fg(Color::White)))
            }
        })
        .collect();

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, frame.area());

    render_bottom_bar(frame, inner, state.paused);

    if state.search_active {
        render_search_bar(frame, inner, &state.search);
    }
}

fn render_bottom_bar(frame: &mut Frame, area: Rect, paused: bool) {
    let bar = Rect {
        x: area.x + 1,
        y: area.y + area.height.saturating_sub(1),
        width: area.width.saturating_sub(2),
        height: 1,
    };
    let text = if paused {
        if area.width >= 55 {
            "  r resume   / find   T:timestamps   ↑↓/k j line   PgUp/PgDn page   g/G top/bottom   s:export   Esc back"
        } else if area.width >= 40 {
            "  r resume   / find   T:timestamps   PgUp/PgDn page   s:export   Esc back"
        } else {
            "  r resume   / find   T:timestamps   s:export   Esc back"
        }
    } else {
        if area.width >= 55 {
            "  p pause   / find   T:timestamps   ↑↓/k j line   PgUp/PgDn page   g/G top/bottom   s:export   Esc back"
        } else if area.width >= 40 {
            "  p pause   / find   T:timestamps   PgUp/PgDn page   s:export   Esc back"
        } else {
            "  p pause   / find   T:timestamps   s:export   Esc back"
        }
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
        "/  search..."
    } else {
        search
    };
    frame.render_widget(
        Paragraph::new(format!("/{}", display))
            .style(Style::default().fg(Color::White).bg(Color::DarkGray)),
        search_area,
    );
}
