use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, BorderType, Row, Table, TableState};

use crate::app::state::VolumesState;

pub fn render(frame: &mut Frame, state: &VolumesState) {
    let area = frame.area();

    let title = if state.loading {
        " VOLUMES (loading...) ".to_string()
    } else {
        format!(" VOLUMES ({}) ", state.items.len())
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);

    if state.items.is_empty() && !state.loading {
        let text = Text::from(vec![
            Line::from(Span::styled("  No volumes found", Style::default().fg(Color::Yellow))),
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

    let widths = [
        Constraint::Length(22),
        Constraint::Length(10),
        Constraint::Fill(1),
    ];

    let header_cells = ["NAME", "DRIVER", "MOUNTPOINT"]
        .iter()
        .map(|h| Cell::from(*h).style(header_style));
    let header_row = Row::new(header_cells).height(1);

    let rows: Vec<Row> = state
        .items
        .iter()
        .enumerate()
        .map(|(idx, v)| {
            let is_selected = state.selected == idx;
            let indicator = if is_selected { "▶" } else { " " };
            let row_style = if is_selected { selected_bg } else { Style::default() };

            Row::new(vec![
                Cell::from(format!("{} {}", indicator, &v.name)),
                Cell::from(v.driver.clone()),
                Cell::from(v.mountpoint.clone()),
            ])
            .style(row_style)
            .height(1)
        })
        .collect();

    let table = Table::new(rows, widths)
        .header(header_row)
        .block(block);

    let mut table_state = TableState::new().with_selected(state.selected);
    frame.render_stateful_widget(table, area, &mut table_state);

    render_footer(frame, inner);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let footer = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(" d  delete  j/k  navigate  Esc  back ")
            .style(Style::default().fg(Color::DarkGray)),
        footer,
    );
}
