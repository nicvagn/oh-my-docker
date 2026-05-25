use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, BorderType, Row, Table, TableState};

use crate::app::state::ContainersState;

pub fn render(frame: &mut Frame, state: &ContainersState, tick_count: u64) {
    let area = frame.area();

    let title = if state.loading {
        " CONTAINERS (loading...) ".to_string()
    } else if !state.filter.is_empty() {
        format!(" CONTAINERS ({}) FILTER '{}' ", state.filtered.len(), state.filter)
    } else {
        format!(" CONTAINERS ({}) ", state.filtered.len())
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);

    if state.loading && !state.docker_connected {
        let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠏'];
        let spinner = spinner_chars[(tick_count as usize / 2) % spinner_chars.len()];
        let text = Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("  {} Connecting to Docker...", spinner),
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    if state.docker_connected && state.items.is_empty() && !state.loading {
        let text = Text::from(vec![
            Line::from(Span::styled("  No containers found", Style::default().fg(Color::Yellow))),
            Line::from(""),
            Line::from(Span::styled("  j/k  navigate  Enter  details  /  search  l  logs  s  shell  ?  help", Style::default().fg(Color::DarkGray))),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    if !state.docker_connected && !state.loading {
        let (msg, hint) = if state.docker_reconnecting {
            ("  Docker daemon not available — reconnecting...", "  Waiting for Docker to come back online")
        } else {
            ("  Docker daemon not available", "  Start Docker and restart the app")
        };
        let color = if state.docker_reconnecting { Color::Yellow } else { Color::Red };
        let spinner = if state.docker_reconnecting { " ⠋" } else { "" };
        let text = Text::from(vec![
            Line::from(Span::styled(format!("{}{}", spinner, msg), Style::default().fg(color))),
            Line::from(""),
            Line::from(Span::styled(hint, Style::default().fg(Color::DarkGray))),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    let header_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let selected_bg = Style::default().bg(Color::Blue).fg(Color::White);

    let widths = [
        Constraint::Min(15),
        Constraint::Min(15),
        Constraint::Length(10),
        Constraint::Fill(1),
        Constraint::Length(12),
    ];

    let header_cells = ["NAME", "IMAGE", "STATUS", "PORTS", "UPTIME"]
        .iter()
        .map(|h| Cell::from(*h).style(header_style));
    let header_row = Row::new(header_cells).height(1);

   let rows: Vec<Row> = state
        .filtered
        .iter()
        .map(|&idx| {
            let c = &state.items[idx];
            let is_selected = state.filtered.get(state.selected) == Some(&idx);
            let is_stopping = state.stopping_containers.contains(&c.id);
            let is_deleting = state.deleting_containers.contains(&c.id);

            let status_color = if is_stopping || is_deleting {
                Color::Yellow
            } else {
                match c.state.as_str() {
                    "running" => Color::Green,
                    "exited" | "dead" => Color::Red,
                    _ => Color::Yellow,
                }
            };

            let status_text = if is_stopping {
                "stopping...".to_string()
            } else if is_deleting {
                "deleting...".to_string()
            } else {
                c.state.clone()
            };

            let uptime_text = if is_deleting {
                "deleting...".to_string()
            } else if is_stopping {
                "stopping...".to_string()
            } else {
                c.status.clone()
            };

            let indicator = if is_selected { "▶" } else { " " };

            let name_cell = Cell::from(format!("{} {}", indicator, &c.name));
            let image_cell = Cell::from(c.image.clone());
            let state_cell = Cell::from(status_text).style(Style::default().fg(status_color));
            let ports_cell = Cell::from(c.ports.clone());
            let uptime_cell = Cell::from(uptime_text);

            let row_style = if is_selected { selected_bg } else { Style::default() };

            Row::new(vec![name_cell, image_cell, state_cell, ports_cell, uptime_cell])
                .style(row_style)
                .height(1)
        })
        .collect();

    let table = Table::new(rows, widths)
        .header(header_row)
        .block(block);

    let mut table_state = TableState::new().with_selected(state.selected);
    frame.render_stateful_widget(table, area, &mut table_state);

    if state.filter_active {
        crate::ui::render_filter_bar(frame, inner, &state.filter, "search");
    }

    render_footer(frame, area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let footer = Rect {
        x: area.x,
        y: area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(" j/k ↓↑  /search  Enter:details  l:logs  s:shell  r:restart  t:stop/start  d:delete  ?:help ")
            .style(Style::default().fg(Color::DarkGray)),
        footer,
    );
}


