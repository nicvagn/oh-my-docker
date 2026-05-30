use std::collections::HashMap;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, BorderType, Row, Table, TableState};

use crate::app::state::ContainersState;
use crate::config::ContainerColumns;


fn project_group_header(group_name: &str, count: usize, selection_mode: bool) -> Row<'static> {
    let header = format!(" {} ({}) ", group_name, count);
    let mut cells: Vec<Cell> = Vec::new();
    if selection_mode {
        cells.push(Cell::from(""));
    }
    cells.push(Cell::from(header).style(Style::default().fg(Color::Yellow)));
    Row::new(cells)
}

pub fn render(frame: &mut Frame, area: Rect, state: &mut ContainersState, tick_count: u64, columns: &ContainerColumns, polling_intervals_ms: u64) {

    let (indicator_char, indicator_color) = if state.loading {
        ('⠋', Color::Yellow)
    } else if !state.docker_connected {
        ('?', Color::Red)
    } else {
        crate::ui::staleness_indicator(state.last_updated, polling_intervals_ms)
    };

    let title = if state.loading {
        format!(" CONTAINERS {} (loading...) ", indicator_char)
    } else if !state.filter.is_empty() {
        format!(" CONTAINERS {} ({}/{}) FILTER '{}' ", indicator_char, state.filtered.len(), state.items.len(), state.filter)
    } else if state.selection_mode && !state.selected_ids.is_empty() {
        format!(" CONTAINERS {} ({}) [{}] ", indicator_char, state.filtered.len(), state.selected_ids.len())
    } else {
        let status_tag = if state.status_filter.is_empty() {
            String::new()
        } else {
            format!(" [{}]", state.status_filter)
        };
        format!(" CONTAINERS {} ({}){} ", indicator_char, state.filtered.len(), status_tag)
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(indicator_color));

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

    if state.show_column_picker {
        render_column_picker(frame, area, columns, state.column_picker_selection);
        return;
    }

    let header_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let selected_bg = Style::default().bg(Color::Blue).fg(Color::White);

    let mut widths = Vec::new();
    let mut header_cells = Vec::new();

    if state.selection_mode {
        widths.push(Constraint::Length(3));
        header_cells.push("");
    }

    if columns.show_name {
        widths.push(Constraint::Min(15));
        header_cells.push("NAME");
    }
    if columns.show_image {
        widths.push(Constraint::Min(14));
        header_cells.push("IMAGE");
    }
    if columns.show_state {
        widths.push(Constraint::Min(16));
        header_cells.push("STATE");
    }
    if columns.show_ports {
        widths.push(Constraint::Fill(1));
        header_cells.push("PORTS");
    }

    let header_row = Row::new(
        header_cells.iter().map(|h| Cell::from(*h).style(header_style))
    ).height(1);

    // Always group by project
    let mut grouped: HashMap<String, Vec<usize>> = HashMap::new();
    for &idx in &state.filtered {
        let project = &state.items[idx].project;
        grouped.entry(if project.is_empty() { "Ungrouped".to_string() } else { project.clone() }).or_default().push(idx);
    }
    let mut selected_row = 0;
    let all_rows: Vec<Row> = {
        let mut rows = Vec::new();
        let mut group_names: Vec<String> = grouped.keys().cloned().collect();
        group_names.sort();
        for group_name in group_names {
            let indices = &grouped[&group_name];
            rows.push(project_group_header(&group_name, indices.len(), state.selection_mode));
            for &idx in indices {
                let is_selected = state.filtered.get(state.selected) == Some(&idx);
                if is_selected {
                    selected_row = rows.len();
                }
                let c = &state.items[idx];
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

                let health_indicator = if !c.health.is_empty() {
                    match c.health.as_str() {
                        "healthy" => ("● ", Color::Green),
                        "unhealthy" => ("✗ ", Color::Red),
                        "starting" => ("◐ ", Color::Yellow),
                        _ => ("", Color::DarkGray),
                    }
                } else {
                    ("", Color::DarkGray)
                };

                let state_text = if is_stopping {
                    format!("{}stopping...", health_indicator.0)
                } else if is_starting {
                    format!("{}starting...", health_indicator.0)
                } else if is_deleting {
                    format!("{}deleting...", health_indicator.0)
                } else {
                    format!("{}{}", health_indicator.0, c.status)
                };

                let indicator = if is_selected { "▶" } else { " " };

                let mut cells: Vec<Cell> = Vec::new();
                if state.selection_mode {
                    let check = if is_id_selected { "[x]" } else { "[ ]" };
                    cells.push(Cell::from(check));
                }
                if columns.show_name {
                    let name_display = format!("{} {}", indicator, &c.name);
                    cells.push(Cell::from(name_display));
                }
                if columns.show_image {
                    cells.push(Cell::from(c.image.clone()));
                }
                if columns.show_state {
                    let indicator_color = if !c.health.is_empty() {
                        match c.health.as_str() {
                            "healthy" => Color::Green,
                            "unhealthy" => Color::Red,
                            "starting" => Color::Yellow,
                            _ => Color::DarkGray,
                        }
                    } else {
                        state_color
                    };
                    cells.push(Cell::from(state_text).style(Style::default().fg(indicator_color)));
                }
                if columns.show_ports {
                    cells.push(Cell::from(c.ports.clone()));
                }

                let row_style = if is_selected { selected_bg } else { Style::default() };
                rows.push(Row::new(cells).style(row_style).height(1));
            }
        }
        rows
    };

    let table = Table::new(all_rows, widths)
        .header(header_row)
        .block(block);

    let mut table_state = TableState::new()
        .with_selected(Some(selected_row))
        .with_offset(state.scroll_offset);
    frame.render_stateful_widget(table, area, &mut table_state);
    state.scroll_offset = table_state.offset();

    if state.filter_active {
        crate::ui::render_filter_bar(frame, inner, &state.filter, "filter");
    }

}

fn render_column_picker(frame: &mut Frame, area: Rect, columns: &ContainerColumns, selection: usize) {
    crate::ui::column_picker::render_column_picker(frame, area, &[
        ("Name", columns.show_name),
        ("Image", columns.show_image),
        ("State", columns.show_state),
        ("Ports", columns.show_ports),
    ], selection);
}


