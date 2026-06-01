use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};

use crate::app::state::{DiagnosticsPhase, DiagnosticsState};

pub fn render(frame: &mut Frame, area: Rect, diagnostics: &mut DiagnosticsState) {
    let block = Block::default()
        .title(format!(" AI DIAGNOSTICS — {} ", &diagnostics.container_id))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);

    if inner.width < 40 {
        render_single_column(frame, inner, diagnostics, block);
    } else {
        render_split_columns(frame, inner, diagnostics, block);
    }
}

fn render_single_column(frame: &mut Frame, area: Rect, diagnostics: &mut DiagnosticsState, block: Block<'_>) {
    let text = build_single_text(diagnostics);
    let max_offset = text.height().saturating_sub(area.height as usize);
    let scroll_offset = diagnostics.scroll_offset.min(max_offset);
    diagnostics.scroll_offset = scroll_offset;

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::White))
        .scroll((scroll_offset as u16, 0))
        .block(block)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn render_split_columns(frame: &mut Frame, area: Rect, diagnostics: &mut DiagnosticsState, block: Block<'_>) {
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 2),
            Constraint::Ratio(1, 2),
        ])
        .split(inner_area);

    let phase = &diagnostics.phase;

    let left_text = build_analysis_text(diagnostics, phase);
    let right_text = build_playbook_text(diagnostics, phase);

    let max_offset_left = left_text.height().saturating_sub(columns[0].height as usize);
    let scroll_left = diagnostics.scroll_offset.min(max_offset_left);
    diagnostics.scroll_offset = scroll_left;

    let left_block = Block::default()
        .title(" ANALYSIS ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Color::Yellow));

    let left_para = Paragraph::new(left_text)
        .style(Style::default().fg(Color::White))
        .scroll((scroll_left as u16, 0))
        .block(left_block)
        .wrap(Wrap { trim: true });

    frame.render_widget(left_para, columns[0]);

    let right_block = Block::default()
        .title(" REPAIR PLAYBOOK ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(Color::Green));

    let right_para = Paragraph::new(right_text)
        .style(Style::default().fg(Color::White))
        .scroll((scroll_left as u16, 0))
        .block(right_block)
        .wrap(Wrap { trim: true });

    frame.render_widget(right_para, columns[1]);
}

fn build_single_text(diagnostics: &DiagnosticsState) -> Text<'static> {
    let phase = &diagnostics.phase;
    let mut lines: Vec<Line<'static>> = vec![];

    match phase {
        DiagnosticsPhase::Collecting => {
            lines.push(Line::from(Span::styled("  Collecting diagnostic data...", Style::default().fg(Color::Yellow))));
        }
        DiagnosticsPhase::Analyzing => {
            lines.push(Line::from(Span::styled("  Sending context to LLM for analysis...", Style::default().fg(Color::Yellow))));
            if !diagnostics.analysis.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled("  --- ANALYSIS ---", Style::default().fg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD))));
                for line in diagnostics.analysis.lines() {
                    lines.push(Line::from(Span::styled(format!("  {}", line), Style::default().fg(Color::White))));
                }
            }
            if !diagnostics.playbook.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled("  --- PLAYBOOK ---", Style::default().fg(Color::Green).add_modifier(ratatui::style::Modifier::BOLD))));
                for line in diagnostics.playbook.lines() {
                    lines.push(Line::from(Span::styled(format!("  {}", line), Style::default().fg(Color::White))));
                }
            }
        }
        DiagnosticsPhase::Done => {
            lines.push(Line::from(Span::styled("  ROOT-CAUSE ANALYSIS", Style::default().fg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD))));
            lines.push(Line::from(""));
            for line in diagnostics.analysis.lines() {
                lines.push(Line::from(Span::styled(format!("  {}", line), Style::default().fg(Color::White))));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("  REPAIR PLAYBOOK", Style::default().fg(Color::Green).add_modifier(ratatui::style::Modifier::BOLD))));
            lines.push(Line::from(""));
            for line in diagnostics.playbook.lines() {
                lines.push(Line::from(Span::styled(format!("  {}", line), Style::default().fg(Color::White))));
            }
        }
        DiagnosticsPhase::Error(msg) => {
            lines.push(Line::from(Span::styled(format!("  Error: {}", msg), Style::default().fg(Color::Red))));
        }
    }
    Text::from(lines)
}

fn build_analysis_text(diagnostics: &DiagnosticsState, phase: &DiagnosticsPhase) -> Text<'static> {
    let mut lines: Vec<Line<'static>> = vec![];
    match phase {
        DiagnosticsPhase::Collecting => {
            lines.push(Line::from(Span::styled("Gathering container context...", Style::default().fg(Color::Yellow))));
        }
        DiagnosticsPhase::Analyzing => {
            if diagnostics.analysis.is_empty() {
                lines.push(Line::from(Span::styled("Waiting for AI analysis...", Style::default().fg(Color::Yellow))));
            } else {
                for line in diagnostics.analysis.lines() {
                    let styled = highlight_severity(line);
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        styled,
                    ]));
                }
            }
        }
        DiagnosticsPhase::Done => {
            if diagnostics.analysis.is_empty() {
                lines.push(Line::from(Span::styled("No issues detected.", Style::default().fg(Color::Green))));
            } else {
                for line in diagnostics.analysis.lines() {
                    let styled = highlight_severity(line);
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        styled,
                    ]));
                }
            }
        }
        DiagnosticsPhase::Error(msg) => {
            lines.push(Line::from(Span::styled(msg.clone(), Style::default().fg(Color::Red))));
        }
    }
    Text::from(lines)
}

fn build_playbook_text(diagnostics: &DiagnosticsState, phase: &DiagnosticsPhase) -> Text<'static> {
    let mut lines: Vec<Line<'static>> = vec![];
    match phase {
        DiagnosticsPhase::Collecting => {
            lines.push(Line::from(Span::styled("Preparing...", Style::default().fg(Color::Yellow))));
        }
        DiagnosticsPhase::Analyzing => {
            if diagnostics.playbook.is_empty() {
                lines.push(Line::from(Span::styled("Waiting for repair steps...", Style::default().fg(Color::Yellow))));
            } else {
                for line in diagnostics.playbook.lines() {
                    lines.push(Line::from(Span::styled(line.to_string(), Style::default().fg(Color::White))));
                }
            }
        }
        DiagnosticsPhase::Done => {
            if diagnostics.playbook.is_empty() {
                lines.push(Line::from(Span::styled("No repair steps needed.", Style::default().fg(Color::Green))));
            } else {
                for line in diagnostics.playbook.lines() {
                    lines.push(Line::from(Span::styled(line.to_string(), Style::default().fg(Color::White))));
                }
            }
        }
        DiagnosticsPhase::Error(_) => {
            lines.push(Line::from(Span::styled("Playbook unavailable due to error.", Style::default().fg(Color::Red))));
        }
    }
    Text::from(lines)
}

fn highlight_severity(line: &str) -> Span<'static> {
    let lower = line.to_lowercase();
    if lower.contains("critical") || lower.contains("fatal") || lower.contains("panic") || lower.contains("crash") {
        Span::styled(line.to_string(), Style::default().fg(Color::Red).add_modifier(ratatui::style::Modifier::BOLD))
    } else if lower.contains("error") || lower.contains("fail") || lower.contains("unhealthy") {
        Span::styled(line.to_string(), Style::default().fg(Color::Red))
    } else if lower.contains("warning") || lower.contains("warn") || lower.contains("caution") {
        Span::styled(line.to_string(), Style::default().fg(Color::Yellow))
    } else if lower.contains("ok") || lower.contains("healthy") || lower.contains("success") || lower.contains("normal") {
        Span::styled(line.to_string(), Style::default().fg(Color::Green))
    } else if lower.contains("step") || lower.contains("1.") || lower.contains("2.") || lower.contains("3.") || lower.starts_with("- ") || lower.starts_with("* ") {
        Span::styled(line.to_string(), Style::default().fg(Color::Cyan))
    } else {
        Span::styled(line.to_string(), Style::default().fg(Color::White))
    }
}
