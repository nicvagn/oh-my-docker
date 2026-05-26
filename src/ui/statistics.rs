use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, BorderType, Row, Table};

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

pub fn render(frame: &mut Frame, state: &StatisticsState) {
    let area = frame.area();

    let title = if state.loading {
        " STATISTICS (loading...) ".to_string()
    } else if state.items.is_empty() {
        " STATISTICS ".to_string()
    } else {
        format!(" STATISTICS ({}){}", state.items.len(), sort_label(&state.sort_by, state.sort_ascending))
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
            Line::from(Span::styled("  No running containers", Style::default().fg(Color::Yellow))),
            Line::from(""),
            Line::from(Span::styled("  ←/→:sort  t:direction  Esc  back", Style::default().fg(Color::DarkGray))),
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

    let table = Table::new(rows, widths).header(header_row).block(block);
    frame.render_widget(table, area);

    let footer = Rect {
        x: inner.x,
        y: inner.y + inner.height.saturating_sub(1),
        width: inner.width,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(" ←/→:sort  t:direction  Esc  back").style(Style::default().fg(Color::DarkGray)),
        footer,
    );
}
