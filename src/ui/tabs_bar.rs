use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Tabs;
use ratatui::Frame;

use crate::app::event::ContainerSummary;
use crate::app::mode::TAB_TITLES;
use crate::ui::theme;

const TAB_DIVIDER: &str = " ▏ ";

pub fn render(frame: &mut Frame, area: Rect, selected_tab: usize, container_items: &[ContainerSummary]) {
    let inactive = Style::default().fg(theme::muted());

    let running_count = container_items.iter().filter(|c| c.state == "running").count();
    let total_count = container_items.len();

    let titles: Vec<Line> = TAB_TITLES.iter().enumerate().map(|(i, t)| {
        let label = if i == 0 {
            format!(" {} [{}/{}] ", t, running_count, total_count)
        } else {
            format!(" {} ", t)
        };
        Line::from(Span::styled(label, inactive))
    }).collect();

    let tabs = Tabs::new(titles)
        .block(theme::panel_block(Span::styled(
            " Oh My Docker ",
            Style::default().fg(theme::muted()),
        )))
        .select(selected_tab)
        .padding_left("")
        .padding_right("")
        .divider(Span::styled(TAB_DIVIDER, inactive))
        .highlight_style(theme::primary().add_modifier(Modifier::REVERSED));

    frame.render_widget(tabs, area);
}
