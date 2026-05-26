use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, BorderType, Row, Table, TableState};

use crate::app::state::{ImageRunState, ImagesState};

fn field_has_error(run: &ImageRunState, field: usize) -> Option<&str> {
    run.validation_errors.iter().find(|(f, _)| *f == field).map(|(_, msg)| msg.as_str())
}

pub fn render_run(frame: &mut Frame, run: &ImageRunState) {
    let block = Block::default()
        .title(format!(" RUN CONTAINER — {} ", &run.image_id[..12.min(run.image_id.len())]))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(frame.area());
    frame.render_widget(block, frame.area());

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
        8 => "Press 'a' to toggle | true = container removed after stop",
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
        " ↑↓/Tab:next field  Enter:run  Esc:cancel  Type:edit  Bksp:delete  a:auto-remove",
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

pub fn render(frame: &mut Frame, state: &ImagesState) {
    let area = frame.area();

    let title = if state.loading {
        " IMAGES (loading...) ".to_string()
    } else if !state.filter.is_empty() {
        format!(" IMAGES ({}) FILTER '{}' ", state.filtered.len(), state.filter)
    } else {
        format!(" IMAGES ({}) ", state.filtered.len())
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
            Line::from(Span::styled("  No images found", Style::default().fg(Color::Yellow))),
            Line::from(""),
            Line::from(Span::styled("  r  run container  d  remove  /  search  Esc  back", Style::default().fg(Color::DarkGray))),
        ]);
        frame.render_widget(Paragraph::new(text).block(block), area);
        return;
    }

    let selected_bg = Style::default().bg(Color::Blue).fg(Color::White);

    let widths = [
        Constraint::Length(22),
        Constraint::Length(12),
        Constraint::Length(14),
        Constraint::Length(10),
    ];

    let header_cells = ["REPOSITORY", "TAG", "IMAGE ID", "SIZE"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Cyan)));
    let header_row = Row::new(header_cells).height(1);

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

            let row_style = if is_selected { selected_bg } else { Style::default() };

            Row::new(vec![
                Cell::from(format!("{} {}", indicator, img.repository)),
                Cell::from(img.tag.clone()),
                Cell::from(img.id[..12.min(img.id.len())].to_string()),
                Cell::from(size_str),
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

    if state.filter_active {
        crate::ui::render_filter_bar(frame, inner, &state.filter, "search");
    }

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
        Paragraph::new(" r  run container  d  remove  D  dangling  p  prune  /  search  Esc  back ")
            .style(Style::default().fg(Color::DarkGray)),
        footer,
    );
}
