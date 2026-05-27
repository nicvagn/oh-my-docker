use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, BorderType, Row, Table, TableState};

use crate::app::state::NetworksState;

pub fn render(frame: &mut Frame, area: Rect, state: &NetworksState, columns: &crate::config::NetworkColumns) {

    if state.show_column_picker {
        render_column_picker(frame, area, columns, state.column_picker_selection);
        return;
    }

    let (indicator_char, indicator_color) = if state.loading {
        ('⠋', Color::Yellow)
    } else {
        crate::ui::staleness_indicator(state.last_updated, Duration::from_secs(20), Duration::from_secs(50))
    };

    let title = if state.loading {
        format!(" NETWORKS {} (loading...) ", indicator_char)
    } else {
        format!(" NETWORKS {} ({}) ", indicator_char, state.items.len())
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(indicator_color));

    if state.items.is_empty() && !state.loading {
        let text = Text::from(vec![
            Line::from(Span::styled("  No networks found", Style::default().fg(Color::Yellow))),
            Line::from(""),
            Line::from(Span::styled("  Esc  back", Style::default().fg(Color::DarkGray))),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    let header_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let selected_bg = Style::default().bg(Color::Blue).fg(Color::White);

    let mut widths = Vec::new();
    let mut header_cells = Vec::new();

    if columns.show_name {
        widths.push(Constraint::Length(22));
        header_cells.push("NAME");
    }
    if columns.show_id {
        widths.push(Constraint::Length(10));
        header_cells.push("ID");
    }
    if columns.show_driver {
        widths.push(Constraint::Length(10));
        header_cells.push("DRIVER");
    }
    if columns.show_scope {
        widths.push(Constraint::Length(8));
        header_cells.push("SCOPE");
    }
    if columns.show_ipam {
        widths.push(Constraint::Length(18));
        header_cells.push("SUBNET");
        widths.push(Constraint::Length(18));
        header_cells.push("GATEWAY");
    }
    widths.push(Constraint::Length(10));
    header_cells.push("CONTAINERS");

    let header_row = Row::new(
        header_cells.iter().map(|h| Cell::from(*h).style(header_style))
    ).height(1);

    let rows: Vec<Row> = state
        .items
        .iter()
        .enumerate()
        .map(|(idx, n)| {
            let is_selected = state.selected == idx;
            let indicator = if is_selected { "▶" } else { " " };
            let row_style = if is_selected { selected_bg } else { Style::default() };

            let mut cells: Vec<Cell> = Vec::new();
            if columns.show_name {
                cells.push(Cell::from(format!("{} {}", indicator, &n.name)));
            }
            if columns.show_id {
                cells.push(Cell::from(n.id[..12.min(n.id.len())].to_string()));
            }
            if columns.show_driver {
                cells.push(Cell::from(n.driver.clone()));
            }
            if columns.show_scope {
                cells.push(Cell::from(n.scope.clone()));
            }
            if columns.show_ipam {
                cells.push(Cell::from(n.subnet.clone()));
                cells.push(Cell::from(n.gateway.clone()));
            }
            cells.push(Cell::from(n.containers.to_string()));

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

}

fn render_column_picker(frame: &mut Frame, area: Rect, columns: &crate::config::NetworkColumns, selection: usize) {
    crate::ui::column_picker::render_column_picker(frame, area, &[
        ("Name", columns.show_name),
        ("ID", columns.show_id),
        ("Driver", columns.show_driver),
        ("Scope", columns.show_scope),
        ("IPAM", columns.show_ipam),
    ], selection);
}

