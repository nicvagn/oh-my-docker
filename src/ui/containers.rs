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
    } else if state.selection_mode && !state.selected_ids.is_empty() {
        format!(" CONTAINERS ({}) [{}] ", state.filtered.len(), state.selected_ids.len())
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

    let widths = if state.selection_mode {
        vec![
            Constraint::Length(3),
            Constraint::Min(12),
            Constraint::Min(12),
            Constraint::Min(16),
            Constraint::Fill(1),
        ]
    } else {
        vec![
            Constraint::Min(15),
            Constraint::Min(14),
            Constraint::Min(16),
            Constraint::Fill(1),
        ]
    };

    let header_cells: [&str; 5] = if state.selection_mode {
        ["", "NAME", "IMAGE", "STATE", "PORTS"]
    } else {
        ["NAME", "IMAGE", "STATE", "PORTS", ""]
    };
    let header_row = Row::new(
        header_cells.iter().map(|h| Cell::from(*h).style(header_style))
    ).height(1);

   let rows: Vec<Row> = state
        .filtered
        .iter()
        .map(|&idx| {
            let c = &state.items[idx];
            let is_selected = state.filtered.get(state.selected) == Some(&idx);
            let is_stopping = state.stopping_containers.contains(&c.id);
            let is_starting = state.starting_containers.contains(&c.id);
            let is_deleting = state.deleting_containers.contains(&c.id);
            let is_id_selected = state.selected_ids.contains(&c.id);

            let state_color = if is_stopping || is_starting || is_deleting {
                Color::Yellow
            } else {
                match c.state.as_str() {
                    "running" => Color::Green,
                    "exited" | "dead" => Color::Red,
                    _ => Color::Yellow,
                }
            };

            let state_text = if is_stopping {
                "stopping...".to_string()
            } else if is_starting {
                "starting...".to_string()
            } else if is_deleting {
                "deleting...".to_string()
            } else {
                c.status.clone()
            };

            let indicator = if is_selected { "▶" } else { " " };

            let cells: Vec<Cell> = if state.selection_mode {
                let check = if is_id_selected { "[x]" } else { "[ ]" };
                vec![
                    Cell::from(check),
                    Cell::from(format!("{} {}", indicator, &c.name)),
                    Cell::from(c.image.clone()),
                    Cell::from(state_text).style(Style::default().fg(state_color)),
                    Cell::from(c.ports.clone()),
                ]
            } else {
                vec![
                    Cell::from(format!("{} {}", indicator, &c.name)),
                    Cell::from(c.image.clone()),
                    Cell::from(state_text).style(Style::default().fg(state_color)),
                    Cell::from(c.ports.clone()),
                ]
            };

            let row_style = if is_selected { selected_bg } else { Style::default() };

            Row::new(cells)
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

    render_footer(frame, inner, state.selection_mode);
}

fn render_footer(frame: &mut Frame, area: Rect, selection_mode: bool) {
    let footer = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };
    let text = if selection_mode {
        " Space:toggle/select  Ctrl+a:all  t:stop  d:delete  Esc:exit mode  j/k ↓↑  /search  Enter:details  l:logs  s:shell  ?:help "
    } else {
        " Space:select mode  j/k ↓↑  /search  Enter:details  l:logs  s:shell  r:restart  t:stop/start  d:delete  ?:help "
    };
    frame.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::DarkGray)),
        footer,
    );
}


