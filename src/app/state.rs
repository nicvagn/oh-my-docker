use std::collections::HashMap;
use std::collections::HashSet;
use tokio::task::AbortHandle;

use crate::config::OmdockerConfig;
use crate::app::event::{ContainerSummary, ImageEntry, LogEntry, DockerEvent, StatEntry, NetworkEntry, VolumeEntry};
use crate::app::mode::ModeStack;

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
    pub buffer: Vec<LogEntry>,
    pub max_lines: usize,
    pub paused: bool,
    pub search: String,
    pub search_active: bool,
    pub scroll_offset: usize,
    pub tail: bool,
    pub show_timestamps: bool,
    pub viewport_height: usize,
}

#[derive(Clone, Debug, Default)]
pub struct ImagesState {
    pub items: Vec<ImageEntry>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub filter: String,
    pub filter_active: bool,
    pub loading: bool,
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
    pub field_focus: usize,
    pub validation_errors: Vec<(usize, String)>,
}

#[derive(Clone, Debug)]
pub struct EventsState {
    pub buffer: Vec<DockerEvent>,
    pub max_events: usize,
    pub filter: String,
    pub filter_active: bool,
    pub scroll_offset: usize,
    pub viewport_height: usize,
}

impl Default for EventsState {
    fn default() -> Self {
        Self {
            buffer: Vec::new(),
            max_events: 10000,
            filter: String::new(),
            filter_active: false,
            scroll_offset: 0,
            viewport_height: 0,
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
}

impl Default for StatisticsState {
    fn default() -> Self {
        Self { items: Vec::new(), loading: true, sort_by: StatSort::Cpu, sort_ascending: false }
    }
}

#[derive(Clone, Debug)]
pub struct NetworksState {
    pub items: Vec<NetworkEntry>,
    pub selected: usize,
    pub loading: bool,
}

impl Default for NetworksState {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            loading: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct VolumesState {
    pub items: Vec<VolumeEntry>,
    pub selected: usize,
    pub loading: bool,
}

impl Default for VolumesState {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            loading: true,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct HelpState {
    pub scroll_offset: usize,
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub mode_stack: ModeStack,
    pub containers: ContainersState,
    pub details: Option<DetailsState>,
    pub logs: Option<LogState>,
    pub images: ImagesState,
    pub image_run: Option<ImageRunState>,
    pub events: EventsState,
    pub shell: Option<ShellState>,
    pub shell_config: Option<ShellConfigState>,
    pub statistics: StatisticsState,
    pub networks: NetworksState,
    pub volumes: VolumesState,
    pub help: HelpState,
    pub config: OmdockerConfig,
    pub update_available: Option<(String, String)>,
    pub error: Option<String>,
    pub error_timer: u8,
    pub error_persistent: bool,
    pub tick_count: u64,
    pub log_streams: HashMap<String, AbortHandle>,
    pub quit: bool,
}

impl AppState {
    pub fn new() -> Self {
       Self {
            mode_stack: ModeStack::new(),
             containers: ContainersState::default(),
            details: None,
            logs: None,
            images: ImagesState::default(),
            image_run: None,
            events: EventsState::default(),
            statistics: StatisticsState::default(),
            networks: NetworksState::default(),
            volumes: VolumesState::default(),
            help: HelpState::default(),
            shell: None,
            shell_config: None,
            config: OmdockerConfig::default(),
            update_available: None,
            error: None,
            error_timer: 0,
            error_persistent: false,
            tick_count: 0,
            log_streams: HashMap::new(),
            quit: false,
        }
    }
}
