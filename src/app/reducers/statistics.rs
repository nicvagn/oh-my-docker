use std::time::Instant;

use crate::app::event::{AppEvent, Command};
use crate::app::state::{AppState, StatSort};

pub fn reduce(state: &mut AppState, event: &AppEvent) -> Vec<Command> {
    match event {
        AppEvent::StatisticsUpdated(stats) => {
            let sort_by = state.statistics.sort_by;
            let ascending = state.statistics.sort_ascending;
            let mut items = stats.clone();
            items.sort_by(|a, b| {
                let cmp = match sort_by {
                    StatSort::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    StatSort::Cpu => a.cpu_percent.partial_cmp(&b.cpu_percent).unwrap_or(std::cmp::Ordering::Equal),
                    StatSort::Memory => a.memory_usage.cmp(&b.memory_usage),
                    StatSort::NetRx => a.net_rx.cmp(&b.net_rx),
                    StatSort::NetTx => a.net_tx.cmp(&b.net_tx),
                    StatSort::BlockRead => a.block_read.cmp(&b.block_read),
                    StatSort::BlockWrite => a.block_write.cmp(&b.block_write),
                    StatSort::Pids => a.pids.cmp(&b.pids),
                };
                if ascending { cmp } else { cmp.reverse() }
            });
            state.statistics.items = items;
            state.statistics.loading = false;
            state.statistics.last_updated = Some(Instant::now());
            state.statistics.scroll_offset = state.statistics.scroll_offset.min(state.statistics.items.len().saturating_sub(1));
        }
        AppEvent::CycleSortStat(dir) => {
            let variants = [
                StatSort::Name,
                StatSort::Cpu,
                StatSort::Memory,
                StatSort::NetRx,
                StatSort::NetTx,
                StatSort::BlockRead,
                StatSort::BlockWrite,
                StatSort::Pids,
            ];
            let current = state.statistics.sort_by;
            let pos = variants.iter().position(|v| *v == current).unwrap_or(0);
            let len = variants.len() as i32;
            let next = (pos as i32 + dir).rem_euclid(len) as usize;
            state.statistics.sort_by = variants[next];
        }
        AppEvent::ToggleSortDirection => {
            state.statistics.sort_ascending = !state.statistics.sort_ascending;
        }
        AppEvent::ScrollStatistics(delta) => {
            let len = state.statistics.items.len();
            if len == 0 {
                state.statistics.scroll_offset = 0;
            } else {
                let max = len.saturating_sub(1);
                if *delta == i32::MAX {
                    state.statistics.scroll_offset = max;
                } else if *delta == i32::MIN {
                    state.statistics.scroll_offset = 0;
                } else {
                    state.statistics.scroll_offset =
                        crate::util::scroll_offset(state.statistics.scroll_offset, *delta, max);
                }
            }
        }
        _ => {}
    }
    Vec::new()
}
