use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState, BorderType};

use crate::app::state::{AppState, ExplorerPanel, ExplorerFocus};

pub fn render(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let container_id = &state.explorer.container_id.clone();

    if container_id.is_empty() {
        render_prompt(frame, area);
        return;
    }

    let show_toast = state.explorer.transfer_message.is_some() || state.explorer.transfer_error.is_some();
    let footer_height: u16 = if show_toast { 3 } else { 1 };
    let main_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: area.height.saturating_sub(footer_height),
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(48),
            Constraint::Percentage(48),
        ])
        .split(main_area);

    let container_name = state.containers.items
        .iter()
        .find(|c| c.id == state.explorer.container_id)
        .map(|c| c.name.as_str())
        .unwrap_or(container_id);

    let current_focus = state.explorer.focus;
    let focus_left = current_focus == ExplorerFocus::Left;

    let host_title = format!("Host ({})", state.explorer.host.path);
    let container_title = format!("{} ({})", container_name, state.explorer.container.path);
    render_panel(frame, chunks[0], &mut state.explorer.host, focus_left, &host_title, state.tick_count);
    render_panel(frame, chunks[1], &mut state.explorer.container, !focus_left, &container_title, state.tick_count);

    let toast_y = area.y + main_area.height;
    if let Some(ref msg) = state.explorer.transfer_message {
        let toast_area = Rect {
            x: area.x,
            y: toast_y,
            width: area.width,
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(msg.clone()).style(Style::default().fg(Color::Black).bg(Color::Green)),
            toast_area,
        );
    }

    if let Some(ref err) = state.explorer.transfer_error {
        let toast_area = Rect {
            x: area.x,
            y: toast_y,
            width: area.width,
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(err.clone()).style(Style::default().fg(Color::Black).bg(Color::Red)),
            toast_area,
        );
    }

    let footer_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };
    render_footer(frame, footer_area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let text = Line::from(vec![
        Span::styled(" Tab:switch ", Style::default().fg(Color::Cyan)),
        Span::styled(" j/k:nav ", Style::default().fg(Color::DarkGray)),
        Span::styled(" Enter:dir ", Style::default().fg(Color::Cyan)),
        Span::styled(" Bksp:up ", Style::default().fg(Color::DarkGray)),
        Span::styled(" PgUp/Dn ", Style::default().fg(Color::Cyan)),
        Span::styled(" g/G:ends ", Style::default().fg(Color::DarkGray)),
        Span::styled(" ^C:copy ", Style::default().fg(Color::Cyan)),
        Span::styled(" d:del ", Style::default().fg(Color::Cyan)),
        Span::styled(" r:ren ", Style::default().fg(Color::Cyan)),
        Span::styled(" R:ref ", Style::default().fg(Color::DarkGray)),
        Span::styled(" /:fil ", Style::default().fg(Color::DarkGray)),
        Span::styled(" Esc ", Style::default().fg(Color::DarkGray)),
    ]);
    frame.render_widget(Paragraph::new(text).style(Style::default().fg(Color::DarkGray)), area);
}

fn render_prompt(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" EXPLORER ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let text = ratatui::text::Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Enter a container ID to start",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Navigate to a container and press 'x'",
            Style::default().fg(Color::DarkGray),
        )),
    ]);

    let paragraph = Paragraph::new(text).style(Style::default().fg(Color::White)).block(block);
    frame.render_widget(paragraph, area);
}

fn render_panel(
    frame: &mut Frame,
    area: Rect,
    panel: &mut ExplorerPanel,
    is_focused: bool,
    panel_title: &str,
    tick_count: u64,
) {
    let border_color = if is_focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let title = if !panel.filter.is_empty() {
        format!(" {} FILTER: {} ", panel_title, panel.filter)
    } else {
        format!(" {} ", panel_title)
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));
    let inner_area = block.inner(area);

    if panel.loading && panel.items.is_empty() {
        let spinner_chars = ['\u{280B}', '\u{2819}', '\u{2839}', '\u{2838}', '\u{283C}', '\u{2834}', '\u{2826}', '\u{2827}', '\u{280F}'];
        let spinner = spinner_chars[(tick_count as usize / 3) % spinner_chars.len()];
        let text = ratatui::text::Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  Loading {}...", spinner),
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), area);
        render_rename_bar(frame, inner_area, panel);
        if panel.filter_active {
            crate::ui::render_filter_bar(frame, inner_area, &panel.filter, "filter");
        }
        return;
    }

    let show_parent = panel.path != "/";

    if panel.items.is_empty() && !panel.loading {
        if show_parent {
            let indicator = if is_focused { "\u{25B6}" } else { " " };
            let style = if is_focused {
                Style::default().fg(Color::White).bg(Color::Blue)
            } else {
                Style::default()
            };
            let mut rows = Vec::new();
            rows.push(Row::new(vec![Cell::from(format!("  {} ..", indicator))])
                .style(style)
                .height(1));
            let table = Table::new(rows, vec![Constraint::Fill(1)])
                .block(block);
            let mut table_state = if is_focused {
                TableState::new().with_selected(Some(0)).with_offset(0)
            } else {
                TableState::new().with_offset(0)
            };
            frame.render_stateful_widget(table, area, &mut table_state);
        } else {
            let text = ratatui::text::Text::from(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  (empty)",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
            ]);
            frame.render_widget(Paragraph::new(text).block(block), area);
        }
        render_rename_bar(frame, inner_area, panel);
        if panel.filter_active {
            crate::ui::render_filter_bar(frame, inner_area, &panel.filter, "filter");
        }
        return;
    }

    let mut rows = Vec::new();

    if show_parent {
        let show_cursor = is_focused && panel.selected == 0;
        let indicator = if show_cursor { "\u{25B6}" } else { " " };
        let style = if show_cursor {
            Style::default().fg(Color::White).bg(Color::Blue)
        } else {
            Style::default()
        };
        rows.push(Row::new(vec![Cell::from(format!("  {} ..", indicator))])
            .style(style)
            .height(1));
    }

    for (i, entry) in panel.items.iter().enumerate() {
        let item_idx = if show_parent { i + 1 } else { i };
        let show_cursor = is_focused && panel.selected == item_idx;
        let indicator = if show_cursor { "\u{25B6}" } else { " " };
        let prefix = if entry.is_dir { "\u{1F4C1}" } else { "\u{1F4C4}" };
        let display = format!("  {} {} {}", indicator, prefix, entry.name);

        let style = if show_cursor {
            Style::default().fg(Color::White).bg(Color::Blue)
        } else {
            Style::default()
        };

        rows.push(Row::new(vec![Cell::from(display)]).style(style).height(1));
    }

    let widths = vec![Constraint::Fill(1)];
    let table = Table::new(rows, widths)
        .block(block);

    let mut table_state = if is_focused {
        let selected_row = panel.selected;
        TableState::new()
            .with_selected(Some(selected_row))
            .with_offset(panel.scroll_offset)
    } else {
        TableState::new().with_offset(panel.scroll_offset)
    };

    frame.render_stateful_widget(table, area, &mut table_state);
    panel.scroll_offset = table_state.offset();
    render_rename_bar(frame, inner_area, panel);
    if panel.filter_active {
        crate::ui::render_filter_bar(frame, inner_area, &panel.filter, "filter");
    }
}

fn render_rename_bar(frame: &mut Frame, inner: Rect, panel: &ExplorerPanel) {
    if !panel.rename_active {
        return;
    }
    let bar_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width.min(50),
        height: 1,
    };
    let display = if panel.rename_buffer.is_empty() {
        "  rename...".to_string()
    } else {
        format!(" {}", panel.rename_buffer)
    };
    frame.render_widget(Clear, bar_area);
    frame.render_widget(
        Paragraph::new(format!("/{}", display))
            .style(Style::default().fg(Color::White).bg(Color::DarkGray)),
        bar_area,
    );
}
