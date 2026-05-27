use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, BorderType, Row, Table, TableState};

use crate::app::state::{ImageRunState, ImagesState};

fn field_has_error(run: &ImageRunState, field: usize) -> Option<&str> {
    run.validation_errors.iter().find(|(f, _)| *f == field).map(|(_, msg)| msg.as_str())
}

pub fn render_run(frame: &mut Frame, area: Rect, run: &ImageRunState) {
    let block = Block::default()
        .title(format!(" RUN CONTAINER — {} ", &run.image_id[..12.min(run.image_id.len())]))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Green));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    // --- Field: Command ---
    let cmd_display = if run.command.is_empty() {
        "(use shell default)".to_string()
    } else {
        run.command.clone()
    };
    lines.push(Line::from(Span::styled(
        format!(" {} Command", if run.field_focus == 0 { "▸" } else { " " }),
        Style::default().fg(if run.field_focus == 0 { Color::White } else { Color::DarkGray }),
    )));
    lines.push(Line::from(Span::styled(
        format!("    {}", cmd_display),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));

    // --- Field: Shell ---
    let shell_display = if run.shell.is_empty() { String::new() } else { run.shell.clone() };
    lines.push(Line::from(Span::styled(
        format!(" {} Shell", if run.field_focus == 1 { "▸" } else { " " }),
        Style::default().fg(if run.field_focus == 1 { Color::White } else { Color::DarkGray }),
    )));
    lines.push(Line::from(Span::styled(
        format!("    {}", shell_display),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));

    // --- Field: User ---
    let user_display = if run.user.is_empty() {
        "(container default)".to_string()
    } else {
        run.user.clone()
    };
    lines.push(Line::from(Span::styled(
        format!(" {} User", if run.field_focus == 2 { "▸" } else { " " }),
        Style::default().fg(if run.field_focus == 2 { Color::White } else { Color::DarkGray }),
    )));
    lines.push(Line::from(Span::styled(
        format!("    {}", user_display),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));

    // --- Field: Workdir ---
    let wd_display = if run.workdir.is_empty() {
        "(container default)".to_string()
    } else {
        run.workdir.clone()
    };
    lines.push(Line::from(Span::styled(
        format!(" {} Workdir", if run.field_focus == 3 { "▸" } else { " " }),
        Style::default().fg(if run.field_focus == 3 { Color::White } else { Color::DarkGray }),
    )));
    lines.push(Line::from(Span::styled(
        format!("    {}", wd_display),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));

    // --- Field: Env Vars ---
    let env_display = if run.env_vars.is_empty() {
        "(none)".to_string()
    } else {
        run.env_vars.clone()
    };
    lines.push(Line::from(Span::styled(
        format!(" {} Env Vars", if run.field_focus == 4 { "▸" } else { " " }),
        Style::default().fg(if run.field_focus == 4 { Color::White } else { Color::DarkGray }),
    )));
    lines.push(Line::from(Span::styled(
        format!("    {}", env_display),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));

    // --- Field: Port Mapping ---
    let port_display = if run.port_mapping.is_empty() {
        "(none)".to_string()
    } else {
        run.port_mapping.clone()
    };
    lines.push(Line::from(Span::styled(
        format!(" {} Port Mapping", if run.field_focus == 5 { "▸" } else { " " }),
        Style::default().fg(if run.field_focus == 5 { Color::White } else { Color::DarkGray }),
    )));
    let port_fg = if field_has_error(run, 5).is_some() { Color::Red } else { Color::Cyan };
    lines.push(Line::from(Span::styled(
        format!("    {}", port_display),
        Style::default().fg(port_fg).add_modifier(Modifier::BOLD),
    )));
    if let Some(err) = field_has_error(run, 5) {
        lines.push(Line::from(Span::styled(
            format!("    ▲ {}", err),
            Style::default().fg(Color::Red),
        )));
    }

    // --- Field: Volumes ---
    let vol_display = if run.volumes.is_empty() {
        "(none)".to_string()
    } else {
        run.volumes.clone()
    };
    lines.push(Line::from(Span::styled(
        format!(" {} Volumes", if run.field_focus == 6 { "▸" } else { " " }),
        Style::default().fg(if run.field_focus == 6 { Color::White } else { Color::DarkGray }),
    )));
    let vol_fg = if field_has_error(run, 6).is_some() { Color::Red } else { Color::Cyan };
    lines.push(Line::from(Span::styled(
        format!("    {}", vol_display),
        Style::default().fg(vol_fg).add_modifier(Modifier::BOLD),
    )));
    if let Some(err) = field_has_error(run, 6) {
        lines.push(Line::from(Span::styled(
            format!("    ▲ {}", err),
            Style::default().fg(Color::Red),
        )));
    }

    // --- Field: Container Name ---
    let name_display = if run.container_name.is_empty() {
        "(auto-generated)".to_string()
    } else {
        run.container_name.clone()
    };
    lines.push(Line::from(Span::styled(
        format!(" {} Container Name", if run.field_focus == 7 { "▸" } else { " " }),
        Style::default().fg(if run.field_focus == 7 { Color::White } else { Color::DarkGray }),
    )));
    lines.push(Line::from(Span::styled(
        format!("    {}", name_display),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));

    // --- Field: Auto Remove ---
    let autoremove_display = if run.autoremove { "true" } else { "false" };
    lines.push(Line::from(Span::styled(
        format!(" {} Auto Remove", if run.field_focus == 8 { "▸" } else { " " }),
        Style::default().fg(if run.field_focus == 8 { Color::White } else { Color::DarkGray }),
    )));
    lines.push(Line::from(Span::styled(
        format!("    {}", autoremove_display),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));

    // Advanced fields
    if run.show_advanced {
        // --- Field: Restart Policy ---
        let restart_display = if run.restart_policy.is_empty() { "(no)" } else { &run.restart_policy };
        lines.push(Line::from(Span::styled(
            format!(" {} Restart Policy", if run.field_focus == 9 { "▸" } else { " " }),
            Style::default().fg(if run.field_focus == 9 { Color::White } else { Color::DarkGray }),
        )));
        lines.push(Line::from(Span::styled(
            format!("    {}", restart_display),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));

        // --- Field: Memory Limit ---
        let mem_display = if run.memory_limit.is_empty() { "(none)" } else { &run.memory_limit };
        lines.push(Line::from(Span::styled(
            format!(" {} Memory Limit", if run.field_focus == 10 { "▸" } else { " " }),
            Style::default().fg(if run.field_focus == 10 { Color::White } else { Color::DarkGray }),
        )));
        lines.push(Line::from(Span::styled(
            format!("    {}", mem_display),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));

        // --- Field: CPU Limit ---
        let cpu_display = if run.cpu_limit.is_empty() { "(none)" } else { &run.cpu_limit };
        lines.push(Line::from(Span::styled(
            format!(" {} CPU Limit", if run.field_focus == 11 { "▸" } else { " " }),
            Style::default().fg(if run.field_focus == 11 { Color::White } else { Color::DarkGray }),
        )));
        lines.push(Line::from(Span::styled(
            format!("    {}", cpu_display),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));

        // --- Field: Network ---
        let net_display = if run.network.is_empty() { "(default)" } else { &run.network };
        lines.push(Line::from(Span::styled(
            format!(" {} Network", if run.field_focus == 12 { "▸" } else { " " }),
            Style::default().fg(if run.field_focus == 12 { Color::White } else { Color::DarkGray }),
        )));
        lines.push(Line::from(Span::styled(
            format!("    {}", net_display),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));

        // --- Field: Labels ---
        let label_display = if run.labels.is_empty() { "(none)" } else { &run.labels };
        lines.push(Line::from(Span::styled(
            format!(" {} Labels", if run.field_focus == 13 { "▸" } else { " " }),
            Style::default().fg(if run.field_focus == 13 { Color::White } else { Color::DarkGray }),
        )));
        lines.push(Line::from(Span::styled(
            format!("    {}", label_display),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));

        // --- Field: Privileged ---
        let priv_display = if run.privileged { "true" } else { "false" };
        lines.push(Line::from(Span::styled(
            format!(" {} Privileged", if run.field_focus == 14 { "▸" } else { " " }),
            Style::default().fg(if run.field_focus == 14 { Color::White } else { Color::DarkGray }),
        )));
        lines.push(Line::from(Span::styled(
            format!("    {}", priv_display),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));
    }

    // Help for current field
    lines.push(Line::from(""));
    let help = match run.field_focus {
        0 => "Empty = use shell | space-separated args like 'sh -c while true; do sleep 3600; done'",
        1 => "Valid values: sh, bash, or a custom path like /bin/zsh",
        2 => "Empty = container default | host = current host user | root | user:group",
        3 => "Empty = container default | / = root | or a custom path like /app",
        4 => "One per line: KEY=value (e.g., 'NODE_ENV=production')",
        5 => "Host:Container or just Container (e.g., '8080:80' or '443')",
        6 => "Host:Container paths, one per line (e.g., '/data:/app/data')",
        7 => "Empty = auto-generated | or a custom name like 'my-container'",
        8 => "Press a to toggle | true = container removed after stop",
        9 => "Restart policy: no, always, on-failure, unless-stopped",
        10 => "Memory limit: 512m, 1g, 256m (empty = no limit)",
        11 => "CPU limit: 0.5, 1, 2 (empty = no limit)",
        12 => "Network name: host, bridge, or custom network (empty = default)",
        13 => "Labels: one per line KEY=value (e.g., 'app=myapp')",
        14 => "Press a to toggle | true = container runs in privileged mode",
        _ => "",
    };
    let val_style = if run.field_focus == 0 && run.command.is_empty() {
        Some("use shell")
    } else if run.field_focus == 1 && run.shell.is_empty() {
        Some("sh")
    } else if (run.field_focus == 2 && run.user.is_empty()) || (run.field_focus == 3 && run.workdir.is_empty()) {
        Some("(default)")
    } else if (run.field_focus == 4 && run.env_vars.is_empty()) || (run.field_focus == 5 && run.port_mapping.is_empty()) || (run.field_focus == 6 && run.volumes.is_empty()) {
        Some("(none)")
    } else if run.field_focus == 7 && run.container_name.is_empty() {
        Some("(auto)")
    } else if run.field_focus == 8 {
        Some(if run.autoremove { "true" } else { "false" })
    } else if run.field_focus == 9 && run.restart_policy.is_empty() {
        Some("no")
    } else if run.field_focus == 10 && run.memory_limit.is_empty() {
        Some("(none)")
    } else if run.field_focus == 11 && run.cpu_limit.is_empty() {
        Some("(none)")
    } else if run.field_focus == 12 && run.network.is_empty() {
        Some("(default)")
    } else if run.field_focus == 13 && run.labels.is_empty() {
        Some("(none)")
    } else if run.field_focus == 14 {
        Some(if run.privileged { "true" } else { "false" })
    } else {
        None
    };
    if let Some(default_label) = val_style {
        lines.push(Line::from(Span::styled(
            format!("  {}  (type or press Enter for {})", help, default_label),
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            format!("  {}", help),
            Style::default().fg(Color::DarkGray),
        )));
    }

    // Error summary
    if !run.validation_errors.is_empty() {
        lines.push(Line::from(""));
        let count = run.validation_errors.len();
        lines.push(Line::from(Span::styled(
            format!("  ⚠ {} error{} — fix before submitting", count, if count == 1 { "" } else { "s" }),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
    }

    // Footer
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " ↑↓ Tab:next field  Enter:run  Esc:cancel  Type:edit  Bksp:delete  a:toggle  ^A:advanced",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(Text::from(lines))
        .style(Style::default().fg(Color::White))
        .block(Block::default());

    frame.render_widget(paragraph, Rect {
        x: inner.x + 1,
        y: inner.y,
        width: inner.width.saturating_sub(2),
        height: inner.height,
    });
}

fn render_column_picker(frame: &mut Frame, area: Rect, columns: &crate::config::ImageColumns, selection: usize) {
    crate::ui::column_picker::render_column_picker(frame, area, &[
        ("Repository", columns.show_repository),
        ("Tag", columns.show_tag),
        ("ID", columns.show_id),
        ("Size", columns.show_size),
    ], selection);
}

pub fn render(frame: &mut Frame, area: Rect, state: &ImagesState, columns: &crate::config::ImageColumns) {
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
        format!(" IMAGES {} (loading...) ", indicator_char)
    } else if !state.filter.is_empty() {
        format!(" IMAGES {} ({}/{}) FILTER '{}' ", indicator_char, state.filtered.len(), state.items.len(), state.filter)
    } else {
        format!(" IMAGES {} ({}) ", indicator_char, state.filtered.len())
    };

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(indicator_color));

    let inner = block.inner(area);

    if state.items.is_empty() && !state.loading {
        let text = Text::from(vec![
            Line::from(Span::styled("  No images found", Style::default().fg(Color::Yellow))),
            Line::from(""),
            Line::from(Span::styled("  r  run container  d  remove  /  filter  Esc  back", Style::default().fg(Color::DarkGray))),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    let selected_bg = Style::default().bg(Color::Blue).fg(Color::White);

    let mut widths = Vec::new();
    let mut header_cells = Vec::new();

    if columns.show_repository {
        widths.push(Constraint::Length(22));
        header_cells.push("REPOSITORY");
    }
    if columns.show_tag {
        widths.push(Constraint::Length(12));
        header_cells.push("TAG");
    }
    if columns.show_id {
        widths.push(Constraint::Length(14));
        header_cells.push("IMAGE ID");
    }
    if columns.show_size {
        widths.push(Constraint::Length(10));
        header_cells.push("SIZE");
    }

    let header_row = Row::new(
        header_cells.iter().map(|h| Cell::from(*h).style(Style::default().fg(Color::Cyan)))
    ).height(1);

    let rows: Vec<Row> = state
        .filtered
        .iter()
        .map(|&idx| {
            let img = &state.items[idx];
            let is_selected = state.filtered.get(state.selected) == Some(&idx);

            let indicator = if is_selected { "▶" } else { " " };
            let size_str = if img.size > 1_000_000_000 {
                format!("{:.1}GB", img.size as f64 / 1_000_000_000.0)
            } else if img.size > 1_000_000 {
                format!("{:.1}MB", img.size as f64 / 1_000_000.0)
            } else if img.size > 1_000 {
                format!("{:.1}KB", img.size as f64 / 1_000.0)
            } else {
                format!("{}B", img.size)
            };

            let mut cells: Vec<Cell> = Vec::new();
            if columns.show_repository {
                cells.push(Cell::from(format!("{} {}", indicator, img.repository)));
            }
            if columns.show_tag {
                cells.push(Cell::from(img.tag.clone()));
            }
            if columns.show_id {
                cells.push(Cell::from(img.id[..12.min(img.id.len())].to_string()));
            }
            if columns.show_size {
                cells.push(Cell::from(size_str));
            }

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
        crate::ui::render_filter_bar(frame, inner, &state.filter, "filter");
    }
}

