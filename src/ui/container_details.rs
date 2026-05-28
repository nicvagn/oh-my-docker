use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, BorderType};

use crate::app::state::{DetailsState, ContainersState};
use crate::ui::theme;
use serde_json::Value;

pub fn render_placeholder(frame: &mut Frame, _area: Rect) {
    let block = Block::default()
        .title(" CONTAINER DETAILS ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::view_border()));

    let text = Text::from(vec![
        Line::from(Span::styled("  No container selected", Style::default().fg(Color::Yellow))),
        Line::from(""),
        Line::from(Span::styled("  Esc  back", Style::default().fg(Color::DarkGray))),
    ]);

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::White))
        .block(block);
    frame.render_widget(paragraph, _area);
}

pub fn render(frame: &mut Frame, area: Rect, details: &mut DetailsState, containers: &ContainersState) {
    let footer_height = 1u16;

    let block = Block::default()
        .title(format!(" CONTAINER DETAILS — {} ", details.container_id))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::view_border()));

    let inner = block.inner(area);
    let content_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: inner.height.saturating_sub(footer_height),
    };
    let footer_area = Rect {
        x: inner.x,
        y: inner.y + content_area.height,
        width: inner.width,
        height: footer_height,
    };

    let text = build_text(details, containers);
    let max_offset = text.height().saturating_sub(content_area.height as usize);
    let scroll_offset = details.scroll_offset.min(max_offset);
    details.scroll_offset = scroll_offset;

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::White))
        .scroll((scroll_offset as u16, 0))
        .block(block);

    frame.render_widget(paragraph, area);

    let footer_spans = vec![
        Span::styled(" l:logs ", Style::default().fg(Color::Cyan)),
        Span::styled(" s:shell ", Style::default().fg(Color::Cyan)),
        Span::styled(" r:restart ", Style::default().fg(Color::Cyan)),
        Span::styled(" t:start/stop ", Style::default().fg(Color::Cyan)),
        Span::styled(" x:explorer ", Style::default().fg(Color::Cyan)),
        Span::styled(" j/k↓↑/PgUp/PgDn scroll ", Style::default().fg(Color::Cyan)),
        Span::styled(" Esc:back ", Style::default().fg(Color::DarkGray)),
    ];

    frame.render_widget(
        Paragraph::new(Line::from(footer_spans))
            .style(Style::default().fg(Color::DarkGray)),
        footer_area,
    );
}

fn build_text(details: &DetailsState, containers: &ContainersState) -> Text<'static> {
    let json_str = match details.json {
        Some(ref s) => s.clone(),
        None => return Text::from(vec![
            Line::from(Span::styled("  Loading container details...", Style::default().fg(Color::Yellow))),
        ]),
    };

    let json: Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return Text::from(vec![
            Line::from(Span::styled("  Failed to parse container JSON", Style::default().fg(Color::Red))),
        ]),
    };

    let is_stopping = containers.stopping_containers.contains(&details.id);
    let is_starting = containers.starting_containers.contains(&details.id);
    let is_deleting = containers.deleting_containers.contains(&details.id);

    let container_state = containers.items.iter().find(|c| c.id == details.id).map(|c| c.state.as_str());

    let mut lines: Vec<Line<'static>> = vec![];

    push_line_name(&mut lines, &json, "Name", "Name");
    push_line(&mut lines, "Image", extract(&json, &["Config", "Image"]));
    push_line(&mut lines, "Command", extract_cmd(&json));
    push_status(&mut lines, &json, container_state, is_starting, is_stopping, is_deleting);
    push_line(&mut lines, "Created", extract(&json, &["Created"]));

    lines.push(Line::from(""));

    push_section_compose(&mut lines, &json);
    push_section_env(&mut lines, &json);
    push_section_volumes(&mut lines, &json);
    push_section_networks(&mut lines, &json);
    push_section_ports(&mut lines, &json);
    push_section_labels(&mut lines, &json);

    Text::from(lines)
}

fn extract(json: &Value, path: &[&str]) -> String {
    let mut current = json;
    for key in path {
        current = current.get(*key).unwrap_or(&Value::Null);
    }
    current.as_str().unwrap_or("").to_string()
}

fn extract_cmd(json: &Value) -> String {
    if let Some(cfg) = json.get("Config") {
        if let Some(cmd) = cfg.get("Cmd").and_then(|v| v.as_array()) {
            let parts: Vec<String> = cmd.iter().filter_map(|v| v.as_str().map(String::from)).collect();
            return parts.join(" ");
        }
    }
    String::new()
}

fn push_line_name(lines: &mut Vec<Line<'static>>, json: &Value, label: &str, key: &str) {
    let val = extract(json, &[key]);
    if !val.is_empty() {
        let display = val.trim_start_matches('/');
        lines.push(Line::from(format!(" {}:         {}", label, display)));
    }
}

fn push_line(lines: &mut Vec<Line<'static>>, label: &str, value: String) {
    if !value.is_empty() {
        lines.push(Line::from(format!(" {}:        {}", label, value)));
    }
}

fn push_status(lines: &mut Vec<Line<'static>>, json: &Value, container_state: Option<&str>, is_starting: bool, is_stopping: bool, is_deleting: bool) {
    let status = if is_deleting {
        "deleting..."
    } else if is_stopping {
        "stopping..."
    } else if is_starting {
        "starting..."
    } else if let Some(state) = container_state {
        state
    } else {
        let s = extract(json, &["State", "Status"]);
        if s.is_empty() {
            return;
        }
        lines.push(Line::from(vec![
            Span::raw(" Status:       "),
            Span::styled(s, Style::default().fg(Color::Yellow)),
        ]));
        return;
    };

    let color = match status {
        "running" => Color::Green,
        "exited" | "dead" => Color::Red,
        _ => Color::Yellow,
    };
    lines.push(Line::from(vec![
        Span::raw(" Status:       "),
        Span::styled(status.to_string(), Style::default().fg(color)),
    ]));
}

fn push_section_env(lines: &mut Vec<Line<'static>>, json: &Value) {
    if let Some(cfg) = json.get("Config") {
        if let Some(env) = cfg.get("Env").and_then(|v| v.as_array()) {
            if !env.is_empty() {
                lines.push(Line::from(Span::styled(" ENVIRONMENT", Style::default().fg(Color::Cyan))));
                for e in env {
                    if let Some(val) = e.as_str() {
                        lines.push(Line::from(format!("  {}", val)));
                    }
                }
                lines.push(Line::from(""));
            }
        }
    }
}

fn push_section_volumes(lines: &mut Vec<Line<'static>>, json: &Value) {
    if let Some(mounts) = json.get("Mounts").and_then(|v| v.as_array()) {
        if !mounts.is_empty() {
            lines.push(Line::from(Span::styled(" VOLUMES", Style::default().fg(Color::Cyan))));
            for m in mounts {
                let src = m.get("Source").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let dst = m.get("Destination").and_then(|v| v.as_str()).unwrap_or("").to_string();
                lines.push(Line::from(format!("  {} → {}", src, dst)));
            }
            lines.push(Line::from(""));
        }
    }
}

fn push_section_networks(lines: &mut Vec<Line<'static>>, json: &Value) {
    if let Some(net) = json.get("NetworkSettings") {
        if let Some(nets) = net.get("Networks").and_then(|v| v.as_object()) {
            if !nets.is_empty() {
                lines.push(Line::from(Span::styled(" NETWORKS", Style::default().fg(Color::Cyan))));
                for (name, settings) in nets {
                    let ip = settings.get("IPAddress").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    lines.push(Line::from(format!("  {} ({})", name, ip)));
                }
                lines.push(Line::from(""));
            }
        }
    }
}

fn push_section_ports(lines: &mut Vec<Line<'static>>, json: &Value) {
    if let Some(net) = json.get("NetworkSettings") {
        if let Some(ports) = net.get("Ports").and_then(|v| v.as_object()) {
            if !ports.is_empty() {
                lines.push(Line::from(Span::styled(" PORTS", Style::default().fg(Color::Cyan))));
                for (container_port, bindings) in ports {
                    if let Some(bindings) = bindings.as_array() {
                        for b in bindings {
                            let host = b.get("HostIp").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let hp = b.get("HostPort").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            lines.push(Line::from(format!("  {}:{} → {}", host, hp, container_port)));
                        }
                    }
                }
                lines.push(Line::from(""));
            }
        }
    }
}

fn push_section_labels(lines: &mut Vec<Line<'static>>, json: &Value) {
    if let Some(cfg) = json.get("Config") {
        if let Some(labels) = cfg.get("Labels").and_then(|v| v.as_object()) {
            if !labels.is_empty() {
                lines.push(Line::from(Span::styled(" LABELS", Style::default().fg(Color::Cyan))));
                for (k, v) in labels {
                    lines.push(Line::from(format!("  {}={}", k, v.as_str().unwrap_or(""))));
                }
                lines.push(Line::from(""));
            }
        }
    }
}

fn push_section_compose(lines: &mut Vec<Line<'static>>, json: &Value) {
    if let Some(cfg) = json.get("Config") {
        if let Some(labels) = cfg.get("Labels").and_then(|v| v.as_object()) {
            let project = labels.get("com.docker.compose.project").and_then(|s| s.as_str()).unwrap_or("");
            let service = labels.get("com.docker.compose.service").and_then(|s| s.as_str()).unwrap_or("");
            let config_files = labels.get("com.docker.compose.config-files").and_then(|s| s.as_str()).unwrap_or("");

            if !project.is_empty() || !service.is_empty() {
                lines.push(Line::from(Span::styled(" COMPOSE", Style::default().fg(Color::Cyan))));
                if !project.is_empty() {
                    lines.push(Line::from(format!("  Project:     {}", project)));
                }
                if !service.is_empty() {
                    lines.push(Line::from(format!("  Service:     {}", service)));
                }
                if !config_files.is_empty() {
                    lines.push(Line::from(format!("  Config:      {}", config_files)));
                }
                lines.push(Line::from(""));
            }
        }
    }
}
