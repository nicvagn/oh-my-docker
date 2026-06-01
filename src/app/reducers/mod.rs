pub mod container;
pub mod diagnostics;
pub mod event;
pub mod explorer;
pub mod image;
pub mod log;
pub mod navigation;
pub mod network;
pub mod shell;
pub mod statistics;
pub mod volume;

pub fn handle_column_nav(name: &str, col_count: usize, selection: &mut usize) {
    match name {
        "next" => *selection = (*selection + 1) % col_count,
        "prev" => *selection = (*selection + col_count - 1) % col_count,
        _ => {}
    }
}
