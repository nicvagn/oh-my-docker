use std::collections::HashMap;
use std::time::Instant;

use ratatui::layout::Constraint;

use crate::app::event::{AppEvent, Command, MouseClickKind};
use crate::app::state::{AppState, ExplorerFocus, StatSort};
use crate::app::mode::{self, Mode, TAB_TITLES};
use crate::ui::resource_panel::{header_column_at, ContainerResource, ImageResource, NetworkResource, VolumeResource, Resource};

fn tab_from_col(col: u16, state: &AppState) -> Option<usize> {
    let padding_right: u16 = 1;
    let divider_len: u16 = 3;
    let mut x: u16 = 1;
    for (i, title) in TAB_TITLES.iter().enumerate() {
        let label = if i == 0 {
            let running = state.containers.items.iter().filter(|c| c.state == "running").count();
            let total = state.containers.items.len();
            format!(" {} [{}/{}] ", title, running, total)
        } else {
            format!(" {} ", title)
        };
        let end = x + label.len() as u16;
        if col >= x && col < end {
            return Some(i);
        }
        x = end + padding_right;
        if i < TAB_TITLES.len() - 1 {
            x += divider_len;
        }
    }
    None
}

fn scroll_state(state: &mut AppState, dir: i32) {
    match state.navigation.mode_stack.current() {
        Mode::Containers => {
            let max = state.containers.filtered.len().saturating_sub(1);
            let new = (state.containers.selected as i32 + dir).clamp(0, max as i32) as usize;
            state.containers.selected = new;
        }
        Mode::Images => {
            let max = state.images.filtered.len().saturating_sub(1);
            let new = (state.images.selected as i32 + dir).clamp(0, max as i32) as usize;
            state.images.selected = new;
        }
        Mode::Networks => {
            let max = state.networks.filtered.len().saturating_sub(1);
            let new = (state.networks.selected as i32 + dir).clamp(0, max as i32) as usize;
            state.networks.selected = new;
        }
        Mode::Volumes => {
            let max = state.volumes.filtered.len().saturating_sub(1);
            let new = (state.volumes.selected as i32 + dir).clamp(0, max as i32) as usize;
            state.volumes.selected = new;
        }
        Mode::Events => {
            let height = state.events.viewport_height.max(1) as i32;
            let max = (state.events.buffer.len() as i32 - height).max(0);
            let new = (state.events.scroll_offset as i32 + dir).clamp(0, max) as usize;
            state.events.scroll_offset = new;
        }
        Mode::Statistics => {
            let len = state.statistics.items.len();
            if len > 0 {
                let max = len.saturating_sub(1);
                let new = (state.statistics.scroll_offset as i32 + dir).clamp(0, max as i32) as usize;
                state.statistics.scroll_offset = new;
            }
        }
        Mode::ContainerDetails(_) => {
            if let Some(ref mut d) = state.navigation.details {
                let new = (d.scroll_offset as i32 + dir).max(0) as usize;
                d.scroll_offset = new;
            }
        }
        Mode::Help => {
            let new = (state.navigation.help.scroll_offset as i32 + dir).max(0) as usize;
            state.navigation.help.scroll_offset = new;
        }
        Mode::Explorer(_) | Mode::ExplorerVolume(_, _) => {
            let panel = match state.explorer.focus {
                ExplorerFocus::Left => &mut state.explorer.host,
                ExplorerFocus::Right => &mut state.explorer.container,
            };
            let len = panel.all_items.len();
            if len > 0 {
                let max = len.saturating_sub(1);
                let new = (panel.scroll_offset as i32 + dir).clamp(0, max as i32) as usize;
                panel.scroll_offset = new;
            }
        }
        _ => {}
    }
}

fn container_filtered_idx_at_visual_row(state: &AppState, absolute_row: u16) -> Option<usize> {
    let all_rows_idx = absolute_row as i32 - 5;
    if all_rows_idx < 0 {
        return None;
    }
    let mut grouped: HashMap<String, Vec<usize>> = HashMap::new();
    for &idx in &state.containers.filtered {
        let project = &state.containers.items[idx].project;
        grouped.entry(if project.is_empty() { "Ungrouped".to_string() } else { project.clone() })
            .or_default()
            .push(idx);
    }
    let mut group_names: Vec<String> = grouped.keys().cloned().collect();
    group_names.sort();
    let mut cur_row = 0usize;
    for group_name in &group_names {
        let indices = &grouped[group_name];
        if cur_row == all_rows_idx as usize {
            return None;
        }
        cur_row += 1;
        for &fi in indices {
            if cur_row == all_rows_idx as usize {
                return state.containers.filtered.iter().position(|&f| f == fi);
            }
            cur_row += 1;
        }
    }
    None
}

fn statistics_column(col: u16, term_width: u16) -> Option<(StatSort, bool)> {
    let inner_col = col.saturating_sub(1);
    let inner_width = term_width.saturating_sub(2);
    let spacing: u16 = 1;
    let lengths: [u16; 5] = [8, 18, 14, 14, 6];
    let lengths_sum: u16 = lengths.iter().sum();
    let col0_w = 10u16.max(inner_width.saturating_sub(lengths_sum + spacing * 5));
    if inner_col < col0_w {
        return Some((StatSort::Name, false));
    }
    let mut accum = col0_w + spacing;
    let sizes = [8, 18, 14, 14, 6];
    let cols = [
        StatSort::Cpu,
        StatSort::Memory,
        StatSort::NetRx,
        StatSort::BlockRead,
        StatSort::Pids,
    ];
    let dual = [false, false, true, true, false];
    for (i, &w) in sizes.iter().enumerate() {
        if inner_col >= accum && inner_col < accum + w {
            return if dual[i] { Some((cols[i], true)) } else { Some((cols[i], false)) };
        }
        accum = accum + w + spacing;
    }
    None
}

fn resort_statistics(state: &mut AppState) {
    let sort_by = state.statistics.sort_by;
    let ascending = state.statistics.sort_ascending;
    state.statistics.items.sort_by(|a, b| {
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
}

pub fn reduce(state: &mut AppState, event: AppEvent) -> Vec<Command> {
    let mut commands = Vec::new();

    match &event {
        // Global events handled inline
        AppEvent::Quit => state.quit = true,

        AppEvent::Tick => {
            state.tick_count = state.tick_count.wrapping_add(1);
            if !state.error_persistent && state.error_timer > 0 {
                state.error_timer -= 1;
                if state.error_timer == 0 {
                    state.error = None;
                    state.error_persistent = false;
                }
            }
            if state.explorer.transfer_message.is_some()
                && state.tick_count >= state.explorer.transfer_message_clear_tick
            {
                state.explorer.transfer_message = None;
            }
            if state.explorer.transfer_error.is_some()
                && state.tick_count >= state.explorer.transfer_error_clear_tick
            {
                state.explorer.transfer_error = None;
            }
        }

        AppEvent::CheckUpdate => {
            if let Some((version, url)) = state.update_available.take() {
                commands.push(Command::DownloadUpdate { version, download_url: url });
                state.error = Some("Downloading update...".to_string());
                state.error_timer = 5;
            } else {
                commands.push(Command::CheckUpdate);
                state.error = Some("Checking for updates...".to_string());
                state.error_timer = 5;
            }
        }
        AppEvent::UpdateAvailable(version, url) => {
            state.update_available = Some((version.clone(), url.clone()));
        }
        AppEvent::Error(msg) => {
            state.error = Some(msg.clone());
            state.error_persistent = true;
        }
        AppEvent::Info(msg) => {
            if msg.is_empty() {
                state.error = None;
                state.error_persistent = false;
                state.error_timer = 0;
            } else {
                state.error = Some(msg.clone());
                state.error_timer = 5;
                state.error_persistent = false;
            }
        }

        AppEvent::ToggleMouse => {
            state.mouse_enabled = !state.mouse_enabled;
            commands.push(Command::ToggleMouseCapture);
        }

        AppEvent::MouseClick { row, col, kind } => {
            let is_base = mode::mode_to_tab(state.navigation.mode_stack.current()).is_some();
            match kind {
                MouseClickKind::ScrollUp => scroll_state(state, -1),
                MouseClickKind::ScrollDown => scroll_state(state, 1),
                MouseClickKind::Left if is_base && *row == 1 => {
                    if let Some(tab) = tab_from_col(*col, state) {
                        state.selected_tab = tab;
                        state.navigation.mode_stack.replace_current(mode::tab_to_mode(tab));
                    }
                }
                MouseClickKind::Left if is_base && *row >= 4 => {
                    let idx = (*row as i32 - 5) as usize;
                    match state.navigation.mode_stack.current() {
                        // --- Header row clicks (sorting) ---
                        Mode::Statistics if *row == 4 => {
                            if let Some((sort, has_secondary)) = statistics_column(*col, state.term_width) {
                                let is_sibling = has_secondary
                                    && matches!(
                                        (sort, state.statistics.sort_by),
                                        (StatSort::NetRx, StatSort::NetTx)
                                            | (StatSort::NetTx, StatSort::NetRx)
                                            | (StatSort::BlockRead, StatSort::BlockWrite)
                                            | (StatSort::BlockWrite, StatSort::BlockRead)
                                    );
                                if sort == state.statistics.sort_by {
                                    if has_secondary {
                                        state.statistics.sort_by = match sort {
                                            StatSort::NetRx => StatSort::NetTx,
                                            StatSort::BlockRead => StatSort::BlockWrite,
                                            _ => sort,
                                        };
                                    } else {
                                        state.statistics.sort_ascending = !state.statistics.sort_ascending;
                                    }
                                } else if is_sibling {
                                    state.statistics.sort_by = sort;
                                    state.statistics.sort_ascending = !state.statistics.sort_ascending;
                                } else {
                                    state.statistics.sort_by = sort;
                                    state.statistics.sort_ascending = true;
                                }
                                resort_statistics(state);
                            }
                        }
                        Mode::Containers if *row == 4 => {
                            let mut constraints = Vec::new();
                            if state.container_extra.selection_mode {
                                constraints.push(Constraint::Length(3));
                            }
                            constraints.extend(ContainerResource::column_headers().into_iter().map(|(_, w)| w));
                            if let Some(col) = header_column_at(*col, state.term_width, &constraints, 1) {
                                let resource_col = col.saturating_sub(if state.container_extra.selection_mode { 1 } else { 0 });
                                state.containers.sort_by_column(resource_col);
                                state.containers.reorder_by_group();
                            }
                        }
                        Mode::Images if *row == 4 => {
                            let constraints: Vec<Constraint> = ImageResource::column_headers().into_iter().map(|(_, w)| w).collect();
                            if let Some(col) = header_column_at(*col, state.term_width, &constraints, 1) {
                                state.images.sort_by_column(col);
                            }
                        }
                        Mode::Networks if *row == 4 => {
                            let constraints: Vec<Constraint> = NetworkResource::column_headers().into_iter().map(|(_, w)| w).collect();
                            if let Some(col) = header_column_at(*col, state.term_width, &constraints, 1) {
                                state.networks.sort_by_column(col);
                            }
                        }
                        Mode::Volumes if *row == 4 => {
                            let constraints: Vec<Constraint> = VolumeResource::column_headers().into_iter().map(|(_, w)| w).collect();
                            if let Some(col) = header_column_at(*col, state.term_width, &constraints, 1) {
                                state.volumes.sort_by_column(col);
                            }
                        }
                        // --- Data row clicks ---
                        Mode::Containers => {
                            if let Some(fi) = container_filtered_idx_at_visual_row(state, *row) {
                                state.containers.selected = fi;
                            }
                        }
                        Mode::Images if idx < state.images.filtered.len() => {
                            state.images.selected = idx;
                        }
                        Mode::Networks if idx < state.networks.filtered.len() => {
                            state.networks.selected = idx;
                        }
                        Mode::Volumes if idx < state.volumes.filtered.len() => {
                            state.volumes.selected = idx;
                        }
                        _ => {}
                    }
                }
                MouseClickKind::Left if matches!(state.navigation.mode_stack.current(), Mode::Explorer(_) | Mode::ExplorerVolume(_, _)) && *row > 0 => {
                    let left_width = (state.term_width * 48) / 100;
                    let is_host = *col < left_width;
                    state.explorer.focus = if is_host { ExplorerFocus::Left } else { ExplorerFocus::Right };
                    let show_parent;
                    let scroll_off;
                    let all_len;
                    {
                        let p = if is_host { &state.explorer.host } else { &state.explorer.container };
                        show_parent = p.path != "/";
                        scroll_off = p.scroll_offset;
                        all_len = p.all_items.len();
                    }
                    let visual_row = row.saturating_sub(1) as usize;
                    let table_row = scroll_off + visual_row;
                    if show_parent && table_row == 0 {
                        state.explorer.last_click_time = None;
                        if is_host {
                            if let Some(parent) = std::path::PathBuf::from(&state.explorer.host.path)
                                .parent().and_then(|p| p.to_str()).filter(|s| !s.is_empty())
                            {
                                state.explorer.host.path = if parent == "/" { "/".to_string() } else { parent.to_string() };
                                state.explorer.host.selected = 0;
                                state.explorer.host.filter = String::new();
                                state.explorer.host.filter_active = false;
                                state.explorer.host.rename_active = false;
                                state.explorer.host.rename_buffer = String::new();
                                commands.push(Command::ListHostDir(state.explorer.host.path.clone()));
                            }
                        } else {
                            let path = &state.explorer.container.path;
                            if path != "/" {
                                let cleaned = path.strip_suffix('/').unwrap_or(path);
                                let parts: Vec<&str> = cleaned.split('/').filter(|s| !s.is_empty()).collect();
                                let new_path = if parts.len() <= 1 {
                                    "/".to_string()
                                } else {
                                    format!("/{}/", parts[..parts.len() - 1].join("/"))
                                };
                                state.explorer.container.path.clone_from(&new_path);
                                state.explorer.container.selected = 0;
                                state.explorer.container.filter = String::new();
                                state.explorer.container.filter_active = false;
                                state.explorer.container.rename_active = false;
                                state.explorer.container.rename_buffer = String::new();
                                let cmd = if let Mode::ExplorerVolume(_, name) = state.navigation.mode_stack.current() {
                                    Command::ListVolumeDir(name.clone(), new_path)
                                } else {
                                    Command::ListContainerDir(state.explorer.container_id.clone(), new_path)
                                };
                                commands.push(cmd);
                            }
                        }
                    } else {
                        let item_idx = if show_parent { table_row.saturating_sub(1) } else { table_row };
                        if item_idx < all_len {
                            let entry_name;
                            let entry_is_dir;
                            {
                                let p = if is_host { &state.explorer.host } else { &state.explorer.container };
                                let entry = &p.all_items[item_idx];
                                entry_name = entry.name.clone();
                                entry_is_dir = entry.is_dir;
                            }
                            let now = Instant::now();
                            let is_double = state.explorer.last_click_time
                                .map(|t| now.saturating_duration_since(t).as_millis() < 300)
                                .unwrap_or(false)
                                && state.explorer.last_click_is_host == is_host
                                && state.explorer.last_click_item_index == item_idx;
                            state.explorer.last_click_time = Some(now);
                            state.explorer.last_click_is_host = is_host;
                            state.explorer.last_click_item_index = item_idx;
                            if is_double && entry_is_dir {
                                if is_host {
                                    let new_path = if state.explorer.host.path == "/" {
                                        format!("/{}", entry_name)
                                    } else {
                                        format!("{}/{}", state.explorer.host.path, entry_name)
                                    };
                                    state.explorer.host.path.clone_from(&new_path);
                                    state.explorer.host.selected = 0;
                                    state.explorer.host.filter = String::new();
                                    state.explorer.host.filter_active = false;
                                    state.explorer.host.rename_active = false;
                                    state.explorer.host.rename_buffer = String::new();
                                    commands.push(Command::ListHostDir(new_path));
                                } else {
                                    let path = &state.explorer.container.path;
                                    let new_path = if path.ends_with('/') {
                                        format!("{}{}", path, entry_name)
                                    } else {
                                        format!("{}/{}", path, entry_name)
                                    };
                                    state.explorer.container.path = format!("{}/", new_path);
                                    state.explorer.container.selected = 0;
                                    state.explorer.container.filter = String::new();
                                    state.explorer.container.filter_active = false;
                                    state.explorer.container.rename_active = false;
                                    state.explorer.container.rename_buffer = String::new();
                                    let cmd = if let Mode::ExplorerVolume(_, name) = state.navigation.mode_stack.current() {
                                        Command::ListVolumeDir(name.clone(), state.explorer.container.path.clone())
                                    } else {
                                        Command::ListContainerDir(state.explorer.container_id.clone(), state.explorer.container.path.clone())
                                    };
                                    commands.push(cmd);
                                }
                            } else {
                                if is_host {
                                    state.explorer.host.selected = table_row;
                                } else {
                                    state.explorer.container.selected = table_row;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        AppEvent::SortByColumn(col) => {
            match state.navigation.mode_stack.current() {
                Mode::Containers => {
                    if *col < ContainerResource::column_headers().len() {
                        state.containers.sort_by_column(*col);
                        state.containers.reorder_by_group();
                    }
                }
                Mode::Images => {
                    if *col < ImageResource::column_headers().len() {
                        state.images.sort_by_column(*col);
                    }
                }
                Mode::Networks => {
                    if *col < NetworkResource::column_headers().len() {
                        state.networks.sort_by_column(*col);
                    }
                }
                Mode::Volumes => {
                    if *col < VolumeResource::column_headers().len() {
                        state.volumes.sort_by_column(*col);
                    }
                }
                _ => {}
            }
        }

        AppEvent::DockerReconnecting => {
            state.container_extra.docker_reconnecting = true;
            state.containers.loading = true;
        }
        AppEvent::DockerReconnected => {
            state.container_extra.docker_reconnecting = false;
            state.container_extra.docker_connected = true;
            state.containers.loading = false;
        }
        AppEvent::DockerConnectionLost(reason) => {
            state.container_extra.docker_connected = false;
            state.container_extra.docker_reconnecting = false;
            state.containers.loading = false;
            state.error = Some(reason.clone());
            state.error_timer = 10;
        }

        // Delegate to sub-reducers
        _ => {
            use crate::app::event::AppEvent;
            match &event {
                AppEvent::Navigate(_)
                | AppEvent::Back
                | AppEvent::ShowHelp
                | AppEvent::HideHelp
                | AppEvent::ScrollHelp(_)
                | AppEvent::ShowConfirmDialog(_, _)
                | AppEvent::ConfirmYes
                | AppEvent::ConfirmNo
                | AppEvent::ShowDetails
                | AppEvent::Inspected(_, _)
                | AppEvent::ScrollDetails(_)
                | AppEvent::FilterSubmit(_)
                | AppEvent::JumpTop
                | AppEvent::JumpBottom => {
                    commands.extend(crate::app::reducers::navigation::reduce(state, &event));
                }
                AppEvent::ContainersUpdated(_)
                | AppEvent::SelectContainer(_)
                | AppEvent::FilterContainers(_)
                | AppEvent::ActivateFilter
                | AppEvent::RestartContainer(_)
                | AppEvent::StopContainer(_)
                | AppEvent::ContainerStopped(_)
                | AppEvent::ContainerStarted(_)
                | AppEvent::StartContainer(_)
                | AppEvent::ContainerDeleted(_)
                | AppEvent::ToggleSelectContainer(_)
                | AppEvent::SelectAllContainers
                | AppEvent::ToggleColumnPicker
                | AppEvent::ToggleColumn(_)
                | AppEvent::BatchToggleContainers(_)
                | AppEvent::CycleStatusFilter => {
                    commands.extend(crate::app::reducers::container::reduce(state, &event));
                }
                AppEvent::ImagesUpdated(_)
                | AppEvent::SelectImage(_)
                | AppEvent::FilterImages(_)
                | AppEvent::ActivateImageFilter
                | AppEvent::PrunedImages(_)
                | AppEvent::RunImage(_, _)
                | AppEvent::ImageRunFieldUpdate(_, _)
                | AppEvent::ImageRunToggleAutoremove
                | AppEvent::ImageRunToggleAdvanced
                | AppEvent::ImageRunFocusNext
                | AppEvent::ImageRunFocusPrev
                | AppEvent::ImageRunSubmit => {
                    commands.extend(crate::app::reducers::image::reduce(state, &event));
                }
                AppEvent::LogLines(_, _)
                | AppEvent::TogglePause
                | AppEvent::ActivateLogSearch
                | AppEvent::SearchLogs(_)
                | AppEvent::SubmitLogSearch
                | AppEvent::ScrollLogs(_)
                | AppEvent::ToggleLogTimestamps
                | AppEvent::ExportLogs(_) => {
                    commands.extend(crate::app::reducers::log::reduce(state, &event));
                }
                AppEvent::EventsUpdated(_)
                | AppEvent::ActivateEventsFilter
                | AppEvent::EventsFilterSubmit
                | AppEvent::FilterEvents(_)
                | AppEvent::ScrollEvents(_) => {
                    commands.extend(crate::app::reducers::event::reduce(state, &event));
                }
                AppEvent::StatisticsUpdated(_)
                | AppEvent::CycleSortStat(_)
                | AppEvent::ScrollStatistics(_) => {
                    commands.extend(crate::app::reducers::statistics::reduce(state, &event));
                }
                AppEvent::ToggleSortDirection => {
                    use crate::app::mode::Mode;
                    match state.navigation.mode_stack.current() {
                        Mode::Statistics => {
                            commands.extend(crate::app::reducers::statistics::reduce(state, &event));
                        }
                        Mode::Containers => {
                            commands.extend(crate::app::reducers::container::reduce(state, &event));
                        }
                        Mode::Images => {
                            commands.extend(crate::app::reducers::image::reduce(state, &event));
                        }
                        Mode::Networks => {
                            commands.extend(crate::app::reducers::network::reduce(state, &event));
                        }
                        Mode::Volumes => {
                            commands.extend(crate::app::reducers::volume::reduce(state, &event));
                        }
                        _ => {}
                    }
                }
                AppEvent::ToggleSelectionMode => {
                    use crate::app::mode::Mode;
                    match state.navigation.mode_stack.current() {
                        Mode::Containers => {
                            commands.extend(crate::app::reducers::container::reduce(state, &event));
                        }
                        Mode::Images => {
                            commands.extend(crate::app::reducers::image::reduce(state, &event));
                        }
                        Mode::Networks => {
                            commands.extend(crate::app::reducers::network::reduce(state, &event));
                        }
                        Mode::Volumes => {
                            commands.extend(crate::app::reducers::volume::reduce(state, &event));
                        }
                        _ => {}
                    }
                }
                AppEvent::ToggleSelectResource(_) | AppEvent::SelectAllResources => {
                    use crate::app::mode::Mode;
                    match state.navigation.mode_stack.current() {
                        Mode::Images => {
                            commands.extend(crate::app::reducers::image::reduce(state, &event));
                        }
                        Mode::Networks => {
                            commands.extend(crate::app::reducers::network::reduce(state, &event));
                        }
                        Mode::Volumes => {
                            commands.extend(crate::app::reducers::volume::reduce(state, &event));
                        }
                        _ => {}
                    }
                }
                AppEvent::NetworksUpdated(_)
                | AppEvent::SelectNetwork(_) => {
                    commands.extend(crate::app::reducers::network::reduce(state, &event));
                }
                AppEvent::VolumesUpdated(_)
                | AppEvent::SelectVolume(_) => {
                    commands.extend(crate::app::reducers::volume::reduce(state, &event));
                }
                AppEvent::CloseShell
                | AppEvent::StartShell(_, _, _, _)
                | AppEvent::ShellConfigSubmit
                | AppEvent::ShellConfigFieldUpdate(_, _)
                | AppEvent::ShellConfigFocusNext
                | AppEvent::ShellConfigFocusPrev => {
                    commands.extend(crate::app::reducers::shell::reduce(state, &event));
                }
                AppEvent::ExplorerSelect
                | AppEvent::ExplorerCopyToContainer
                | AppEvent::ExplorerCopyFromContainer
                | AppEvent::ExplorerTransferComplete(_)
                | AppEvent::ExplorerTransferError(_)
                | AppEvent::ExplorerFilter(_)
                | AppEvent::ExplorerContainerDirUpdated(_, _, _)
                | AppEvent::ExplorerHostGoUp
                | AppEvent::ExplorerContainerGoUp
                | AppEvent::ExplorerHostSelect(_)
                | AppEvent::ExplorerContainerSelect(_)
                | AppEvent::ExplorerHostEnterDir(_)
                | AppEvent::ExplorerContainerEnterDir(_)
                | AppEvent::ExplorerHostRefresh
                | AppEvent::ExplorerContainerRefresh
                | AppEvent::ExplorerHostActivateFilter
                | AppEvent::ExplorerContainerActivateFilter
                | AppEvent::ExplorerFilterSubmit
                | AppEvent::ExplorerHostActivateRename
                | AppEvent::ExplorerContainerActivateRename
                | AppEvent::ExplorerRenameUpdate(_)
                | AppEvent::ExplorerRenameSubmit
                |                 AppEvent::ExplorerHostDirUpdated(_, _)
                | AppEvent::ContainerWorkingDir(_, _) => {
                    commands.extend(crate::app::reducers::explorer::reduce(state, &event));
                }
                AppEvent::StartDiagnostics(_)
                | AppEvent::DiagnosticsPhaseUpdate(_)
                | AppEvent::DiagnosticsChunk(_)
                | AppEvent::DiagnosticsPlaybook(_)
                | AppEvent::DiagnosticsDone
                | AppEvent::DiagnosticsError(_)
                | AppEvent::ScrollDiagnostics(_) => {
                    commands.extend(crate::app::reducers::diagnostics::reduce(state, &event));
                }
                _ => {}
            }
        }
    }

    commands
}
