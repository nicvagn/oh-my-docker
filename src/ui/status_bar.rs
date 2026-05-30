use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::ui::theme;

fn shortcuts(selected_tab: usize) -> &'static str {
    match selected_tab {
        0 => " Enter:details  l:logs  s:shell  /:filter  t:start/stop  r:restart  d:delete  Space:select  S:status  Ctrl+U/D:page  g/G:top/bot ",
        1 => " /:filter  r:run  d:delete  D:dangling  p:prune  Ctrl+U/D:page  g/G:top/bot ",
        2 => " d:delete ",
        3 => " d:delete ",
        4 => " /:filter ",
        5 => " left/right:navigate  t:direction ",
        6 => " Esc:back  arrows:scroll  j/k:scroll  PgUp/PgDn:page  g/G:top/bottom ",
        _ => "",
    }
}

pub fn render(frame: &mut Frame, area: Rect, selected_tab: usize) {
    let base = shortcuts(selected_tab);
    let pad = (area.width as usize).saturating_sub(base.len());
    let text = if pad > 0 {
        format!("{}{}", base, " ".repeat(pad))
    } else {
        base.to_string()
    };
    frame.render_widget(
        Paragraph::new(text).style(theme::status_bar_default()),
        area,
    );
}
