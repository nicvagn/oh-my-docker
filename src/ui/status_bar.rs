use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::mode::Mode;
use crate::ui::theme;

fn shortcuts(mode: &Mode) -> &'static str {
    match mode {
        Mode::Containers => {
            " Enter:details  l:logs  s:shell  /:filter  S:status  Ctrl+U/D:page  g/G:top/bot  t:start/stop  r:restart  d:delete  Space:select "
        }
        Mode::ContainerDetails(_) => {
            " j/k:scroll  PgUp/PgDn:page  g/G:top/bot  l:logs  s:shell  x:explorer  t:start/stop  r:restart  Esc:back "
        }
        Mode::Logs(_) => {
            " j/k:scroll  PgUp/PgDn:page  g/G:top/bot  /:search  p:pause  T:timestamps  s:export  Esc:back "
        }
        Mode::Images => {
            " /:filter  r:run  d:delete  D:dangling  p:prune  Ctrl+U/D:page  g/G:top/bot "
        }
        Mode::ImageRun(_) => {
            " Tab/↓:next  ↑:prev  Enter:run  a:toggle  ^A:advanced  Esc:back "
        }
        Mode::Shell(_) => {
            " Type commands  Ctrl+D/Ctrl+C:exit  Esc:close "
        }
        Mode::ShellConfig(_) => {
            " Tab/↓:next  ↑:prev  Enter:save+launch  Esc:back "
        }
        Mode::Events => {
            " /:filter  j/k:scroll  g/G:top/bot  Esc:back "
        }
        Mode::Statistics => {
            " ←/→:sort field  t:direction  Esc:back "
        }
        Mode::Networks => {
            " d:delete  Esc:back "
        }
        Mode::Volumes => {
            " d:delete  Esc:back "
        }
        Mode::Explorer(_) => {
            " Tab:focus  ↑/↓:nav  Enter:dir  Backspace:up  PgUp/PgDn:page  g/G:top/bot  /:filter  r:rename  R:refresh  d:delete  ^C:copy  Esc:back "
        }
        Mode::Help => {
            " Esc:back  j/k:scroll  PgUp/PgDn:page  g/G:top/bot "
        }
        Mode::ConfirmDialog { .. } => {
            " y:yes  n:no  Enter:yes  Esc:no "
        }
    }
}

pub fn render(frame: &mut Frame, area: Rect, mode: &Mode) {
    let base = shortcuts(mode);
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
