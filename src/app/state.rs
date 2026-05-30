use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::time::Instant;
use tokio::task::AbortHandle;

use crate::config::OmdockerConfig;
use crate::app::event::{ContainerSummary, ImageEntry, LogEntry, DockerEvent, StatEntry, NetworkEntry, VolumeEntry};

use crate::app::mode;
use crate::app::navigation::NavigationState;
use crate::input::keymap::KeyMap;

#[derive(Clone, Debug)]
pub struct ContainersState {
    pub items: Vec<ContainerSummary>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub filter: String,
    pub filter_active: bool,
    pub loading: bool,
    pub docker_connected: bool,
    pub docker_reconnecting: bool,
    pub stopping_containers: HashSet<String>,
    pub starting_containers: HashSet<String>,
    pub deleting_containers: HashSet<String>,
    pub selection_mode: bool,
    pub selected_ids: HashSet<String>,
    pub last_updated: Option<Instant>,
    pub show_column_picker: bool,
    pub column_picker_selection: usize,
    pub scroll_offset: usize,
    pub status_filter: String,
}

impl Default for ContainersState {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            filtered: Vec::new(),
            selected: 0,
            filter: String::new(),
            filter_active: false,
            loading: true,
            docker_connected: false,
            docker_reconnecting: false,
            stopping_containers: HashSet::new(),
            starting_containers: HashSet::new(),
            deleting_containers: HashSet::new(),
            selection_mode: false,
            selected_ids: HashSet::new(),
            last_updated: None,
            show_column_picker: false,
            column_picker_selection: 0,
            scroll_offset: 0,
            status_filter: String::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DetailsState {
    pub id: String,
    pub container_id: String,
    pub json: Option<String>,
    pub scroll_offset: usize,
}

#[derive(Clone, Debug)]
pub struct LogState {
    pub container_id: String,
    pub buffer: VecDeque<LogEntry>,
    pub max_lines: usize,
    pub paused: bool,
    pub search: String,
    pub search_active: bool,
    pub scroll_offset: usize,
    pub tail: bool,
    pub show_timestamps: bool,
    pub viewport_height: usize,
}

#[derive(Clone, Debug)]
pub struct ImagesState {
    pub items: Vec<ImageEntry>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub filter: String,
    pub filter_active: bool,
    pub loading: bool,
    pub last_updated: Option<Instant>,
    pub show_column_picker: bool,
    pub column_picker_selection: usize,
    pub scroll_offset: usize,
}

impl Default for ImagesState {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            filtered: Vec::new(),
            selected: 0,
            filter: String::new(),
            filter_active: false,
            loading: true,
            last_updated: None,
            show_column_picker: false,
            column_picker_selection: 0,
            scroll_offset: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ImageRunState {
    pub image_id: String,
    pub command: String,
    pub shell: String,
    pub user: String,
    pub workdir: String,
    pub env_vars: String,
    pub port_mapping: String,
    pub volumes: String,
    pub container_name: String,
    pub autoremove: bool,
    pub restart_policy: String,
    pub memory_limit: String,
    pub cpu_limit: String,
    pub network: String,
    pub labels: String,
    pub privileged: bool,
    pub field_focus: usize,
    pub validation_errors: Vec<(usize, String)>,
    pub show_advanced: bool,
}

#[derive(Clone, Debug)]
pub struct EventsState {
    pub buffer: VecDeque<DockerEvent>,
    pub max_events: usize,
    pub filter: String,
    pub filter_active: bool,
    pub scroll_offset: usize,
    pub viewport_height: usize,
    pub last_updated: Option<Instant>,
}

impl Default for EventsState {
    fn default() -> Self {
        Self {
            buffer: VecDeque::new(),
            max_events: 10000,
            filter: String::new(),
            filter_active: false,
            scroll_offset: 0,
            viewport_height: 0,
            last_updated: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ShellState {
    pub container_id: String,
    pub active: bool,
    pub stop_on_exit: bool,
    pub shell: String,
    pub user: String,
    pub workdir: String,
}

#[derive(Clone, Debug)]
pub struct ShellConfigState {
    pub container_id: String,
    pub shell: String,
    pub user: String,
    pub workdir: String,
    pub field_focus: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum StatSort { Name, Cpu, Memory, NetRx, NetTx, BlockRead, BlockWrite, Pids }

#[derive(Clone, Debug)]
pub struct StatisticsState {
    pub items: Vec<StatEntry>,
    pub loading: bool,
    pub sort_by: StatSort,
    pub sort_ascending: bool,
    pub last_updated: Option<Instant>,
}

impl Default for StatisticsState {
    fn default() -> Self {
        Self { items: Vec::new(), loading: true, sort_by: StatSort::Cpu, sort_ascending: false, last_updated: None }
    }
}

#[derive(Clone, Debug)]
pub struct NetworksState {
    pub items: Vec<NetworkEntry>,
    pub selected: usize,
    pub loading: bool,
    pub last_updated: Option<Instant>,
    pub show_column_picker: bool,
    pub column_picker_selection: usize,
    pub scroll_offset: usize,
}

impl Default for NetworksState {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            loading: true,
            last_updated: None,
            show_column_picker: false,
            column_picker_selection: 0,
            scroll_offset: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct VolumesState {
    pub items: Vec<VolumeEntry>,
    pub selected: usize,
    pub loading: bool,
    pub last_updated: Option<Instant>,
    pub show_column_picker: bool,
    pub column_picker_selection: usize,
    pub scroll_offset: usize,
}

impl Default for VolumesState {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            loading: true,
            last_updated: None,
            show_column_picker: false,
            column_picker_selection: 0,
            scroll_offset: 0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct HelpState {
    pub scroll_offset: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExplorerFocus {
    Left,
    Right,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct ExplorerEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: String,
    pub permissions: String,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct ExplorerPanel {
    pub path: String,
    pub items: Vec<ExplorerEntry>,
    pub all_items: Vec<ExplorerEntry>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub filter: String,
    pub filter_active: bool,
    pub rename_active: bool,
    pub rename_buffer: String,
    pub loading: bool,
}

impl Default for ExplorerPanel {
    fn default() -> Self {
        Self {
            path: String::new(),
            items: Vec::new(),
            all_items: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            filter: String::new(),
            filter_active: false,
            rename_active: false,
            rename_buffer: String::new(),
            loading: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExplorerState {
    pub focus: ExplorerFocus,
    pub host: ExplorerPanel,
    pub container: ExplorerPanel,
    pub container_id: String,
    pub transfer_message: Option<String>,
    pub transfer_error: Option<String>,
    pub transfer_message_clear_tick: u64,
    pub transfer_error_clear_tick: u64,
}

impl Default for ExplorerState {
    fn default() -> Self {
        Self {
            focus: ExplorerFocus::Left,
            host: ExplorerPanel::default(),
            container: ExplorerPanel::default(),
            container_id: String::new(),
            transfer_message: None,
            transfer_error: None,
            transfer_message_clear_tick: 0,
            transfer_error_clear_tick: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub navigation: NavigationState,
    pub containers: ContainersState,
    pub images: ImagesState,
    pub events: EventsState,
    pub statistics: StatisticsState,
    pub networks: NetworksState,
    pub volumes: VolumesState,
    pub explorer: ExplorerState,
    pub config: OmdockerConfig,
    pub keymap: KeyMap,
    pub update_available: Option<(String, String)>,
    pub error: Option<String>,
    pub error_timer: u8,
    pub error_persistent: bool,
    pub selected_tab: usize,
    pub previous_tab: usize,
    pub tick_count: u64,
    pub log_streams: HashMap<String, AbortHandle>,
    pub quit: bool,
}

impl AppState {
    pub fn new() -> Self {
       Self {
            navigation: NavigationState::new(),
             containers: ContainersState::default(),
            images: ImagesState::default(),
            events: EventsState::default(),
            statistics: StatisticsState::default(),
            networks: NetworksState::default(),
            volumes: VolumesState::default(),
            explorer: ExplorerState::default(),
            selected_tab: mode::mode_to_tab(&mode::Mode::Containers).unwrap_or(0),
            previous_tab: 0,
            config: OmdockerConfig::default(),
            keymap: KeyMap::default(),
            update_available: None,
            error: None,
            error_timer: 0,
            error_persistent: false,
            tick_count: 0,
            log_streams: HashMap::new(),
            quit: false,
        }
    }

    pub fn rebuild_keymap(&mut self) {
        self.keymap = self.config.keymap();
    }
}
