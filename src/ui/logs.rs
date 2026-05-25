use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, BorderType};

use crate::app::state::LogState;

pub fn render(frame: &mut Frame, state: &LogState) {
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

    let start = state.buffer.len().saturating_sub(height + state.scroll_offset);
    let lines: Vec<Line> = state
        .buffer
        .iter()
        .skip(start)
        .take(height)
        .map(|entry| {
            let text = entry.message.trim_end().to_string();

            if has_filter && text.to_lowercase().contains(&state.search.to_lowercase()) {
                let search_lower = state.search.to_lowercase();
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
                            text[i..i + search_lower.len()].to_string(),
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ));

                        last_end = i + search_lower.len();
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
                        text,
                        Style::default().fg(Color::Yellow).bg(Color::DarkGray),
                    ));
                }

                Line::from(spans)
            } else {
                Line::from(Span::styled(text, Style::default().fg(Color::White)))
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
        if area.width >= 50 {
            "  r resume   / find   ↑↓/k j line   PgUp/PgDn page   g/G top/bottom   s:export   Esc back"
        } else if area.width >= 36 {
            "  r resume   / find   PgUp/PgDn page   s:export   Esc back"
        } else {
            "  r resume   / find   s:export   Esc back"
        }
    } else {
        if area.width >= 50 {
            "  p pause   / find   ↑↓/k j line   PgUp/PgDn page   g/G top/bottom   s:export   Esc back"
        } else if area.width >= 36 {
            "  p pause   / find   PgUp/PgDn page   s:export   Esc back"
        } else {
            "  p pause   / find   s:export   Esc back"
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
