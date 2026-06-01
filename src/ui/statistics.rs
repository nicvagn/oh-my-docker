use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, BorderType, Row, Table, TableState};

use crate::app::state::{StatSort, StatisticsState};

fn sort_label(sort_by: &StatSort, ascending: bool) -> String {
    let field = match sort_by {
        StatSort::Name => "Name",
        StatSort::Cpu => "CPU %",
        StatSort::Memory => "Memory",
        StatSort::NetRx => "Net Rx",
        StatSort::NetTx => "Net Tx",
        StatSort::BlockRead => "Block Read",
        StatSort::BlockWrite => "Block Write",
        StatSort::Pids => "PIDs",
    };
    let dir = if ascending { " \u{25b4}" } else { " \u{25be}" };
    format!(" (sorted by {}{})", field, dir)
}

pub fn render(frame: &mut Frame, area: Rect, state: &StatisticsState, tick_count: u64) {

    let (indicator_char, indicator_color) = if state.loading {
        (crate::ui::spinner_char(tick_count), Color::Yellow)
    } else {
        let (ch, color) = if let Some(instant) = state.last_updated {
            let elapsed = instant.elapsed();
            let fresh = std::time::Duration::from_secs(4);
            let stale = std::time::Duration::from_secs(10);
            if elapsed < fresh { ('●', Color::Green) }
            else if elapsed < stale { ('○', Color::Yellow) }
            else { ('◌', Color::Red) }
        } else { ('?', Color::DarkGray) };
        (ch, color)
    };

    let title = if state.loading {
        format!(" STATISTICS {} (loading...) ", indicator_char)
    } else if state.items.is_empty() {
        format!(" STATISTICS {} ", indicator_char)
    } else {
        format!(" STATISTICS {} ({}){}", indicator_char, state.items.len(), sort_label(&state.sort_by, state.sort_ascending))
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(indicator_color));

    if state.items.is_empty() && !state.loading {
        let text = Text::from(vec![
            Line::from(Span::styled("  No running containers", Style::default().fg(Color::Yellow))),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    let header_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    let widths = [
        Constraint::Min(10),
        Constraint::Length(8),
        Constraint::Length(18),
        Constraint::Length(14),
        Constraint::Length(14),
        Constraint::Length(6),
    ];

    let current_sort = &state.sort_by;
    let ascending = state.sort_ascending;

    let header_cells = ["NAME", "CPU %", "MEM USAGE/LIMIT", "NET I/O", "BLOCK I/O", "PIDS"]
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let mut cell = Cell::from(*h).style(header_style);
            let is_current = matches!((i, current_sort),
                (0, StatSort::Name) | (1, StatSort::Cpu) | (2, StatSort::Memory)
                | (3, StatSort::NetRx) | (3, StatSort::NetTx)
                | (4, StatSort::BlockRead) | (4, StatSort::BlockWrite)
                | (5, StatSort::Pids));
            if is_current {
                let arrow = if ascending { " \u{25b4}" } else { " \u{25be}" };
                cell = Cell::from(format!("{}{}", h, arrow)).style(header_style.fg(Color::Yellow));
            }
            cell
        });
    let header_row = Row::new(header_cells).height(1);

    let rows: Vec<Row> = state
        .items
        .iter()
        .map(|s| {
            let cpu_color = if s.cpu_percent > 90.0 {
                Color::Red
            } else if s.cpu_percent > 50.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            let mem_pct = s.memory_percent;
            let mem_color = if mem_pct > 90.0 {
                Color::Red
            } else if mem_pct > 50.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            let cpu_str = format!("{:.1}%", s.cpu_percent);
            let mem_str = format!(
                "{:.1}MiB / {:.1}MiB",
                s.memory_usage as f64 / 1_048_576.0,
                s.memory_limit as f64 / 1_048_576.0
            );
            let net_str = format!(
                "{:.1}MB / {:.1}MB",
                s.net_rx as f64 / 1_000_000.0,
                s.net_tx as f64 / 1_000_000.0
            );
            let block_str = if s.block_read > 0 || s.block_write > 0 {
                format!(
                    "{:.1}MB / {:.1}MB",
                    s.block_read as f64 / 1_000_000.0,
                    s.block_write as f64 / 1_000_000.0
                )
            } else {
                "0B / 0B".to_string()
            };

            Row::new(vec![
                Cell::from(format!(" {}", &s.name)),
                Cell::from(cpu_str).style(Style::default().fg(cpu_color)),
                Cell::from(mem_str).style(Style::default().fg(mem_color)),
                Cell::from(net_str),
                Cell::from(block_str),
                Cell::from(s.pids.to_string()),
            ])
            .height(1)
        })
        .collect();

    let inner_height = block.inner(area).height as usize;
    let scroll_offset = state.scroll_offset.min(rows.len().saturating_sub(inner_height));

    let mut table_state = TableState::new().with_offset(scroll_offset);
    let table = Table::new(rows, widths).header(header_row).block(block);
    frame.render_stateful_widget(table, area, &mut table_state);
}
