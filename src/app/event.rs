use crate::app::mode::Mode;

#[derive(Clone, Debug)]
pub struct ContainerSummary {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    pub status: String,
    pub ports: String,
}

#[derive(Clone, Debug)]
pub struct ImageEntry {
    pub id: String,
    pub repository: String,
    pub tag: String,
    pub size: i64,
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct DockerEvent {
    pub timestamp: String,
    pub kind: String,
    pub action: String,
    pub actor: String,
}

#[derive(Clone, Debug)]
pub struct StatEntry {
    #[allow(dead_code)]
    pub container_id: String,
    pub name: String,
    pub cpu_percent: f64,
    pub memory_usage: u64,
    pub memory_limit: u64,
    pub memory_percent: f64,
    pub net_rx: u64,
    pub net_tx: u64,
    pub block_read: u64,
    pub block_write: u64,
    pub pids: u64,
}

#[derive(Clone, Debug)]
pub struct NetworkEntry {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
    pub subnet: String,
    pub gateway: String,
    pub containers: usize,
}

#[derive(Clone, Debug)]
pub struct VolumeEntry {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
}

#[derive(Clone, Debug)]
pub enum ShellConfigField {
    Shell,
    User,
    Workdir,
}

#[derive(Clone, Debug)]
pub enum ImageRunField {
    Command,
    Shell,
    User,
    Workdir,
    EnvVars,
    PortMapping,
    Volumes,
    ContainerName,
}

#[derive(Clone, Debug)]
pub enum ConfirmAction {
    DeleteContainer(String),
    RemoveImage(String),
    RemoveNetwork(String),
    RemoveVolume(String),
}

#[derive(Clone, Debug)]
pub enum AppEvent {
    Navigate(Mode),
    Back,
    Quit,
    Tick,
    ShowHelp,
    HideHelp,
    ScrollHelp(i32),
    Error(String),
    Info(String),

    ShowConfirmDialog(String, ConfirmAction),
    ConfirmYes,
    ConfirmNo,

    DockerReconnecting,
    DockerReconnected,
    DockerConnectionLost(String),

    ContainersUpdated(Vec<ContainerSummary>),
    SelectContainer(usize),
    FilterContainers(String),
    ActivateFilter,
    FilterSubmit(Option<usize>),
    RestartContainer(String),
    StopContainer(String),
    ContainerStopped(String),
    StartContainer(String),
    #[allow(dead_code)]
    DeleteContainer(String),
    ContainerDeleted(String),
    ShowDetails,
    Inspected(serde_json::Value, String),
    ScrollDetails(i32),

    LogLines(String, Vec<LogEntry>),
    TogglePause,
    ActivateLogSearch,
    SearchLogs(String),
    SubmitLogSearch,
    ScrollLogs(i32),
    JumpTop,
    JumpBottom,
    ExportLogs(String),

    CloseShell,
    StartShell(String, String, String, String),

    ShellConfigSubmit,
    ShellConfigFieldUpdate(ShellConfigField, String),
    ShellConfigFocusNext,
    ShellConfigFocusPrev,

    ImagesUpdated(Vec<ImageEntry>),
    SelectImage(usize),
    FilterImages(String),
    ActivateImageFilter,
    #[allow(dead_code)]
    RemoveImage(String),
    RunImage(String),
    ImageRunFieldUpdate(ImageRunField, String),
    ImageRunToggleAutoremove,
    ImageRunFocusNext,
    ImageRunFocusPrev,
    ImageRunSubmit,

    CheckUpdate,
    UpdateAvailable(String, String),

     EventsUpdated(Vec<DockerEvent>),
    ActivateEventsFilter,
    EventsFilterSubmit,
    FilterEvents(String),

    StatisticsUpdated(Vec<StatEntry>),
    NetworksUpdated(Vec<NetworkEntry>),
    SelectNetwork(usize),
    #[allow(dead_code)]
    RemoveNetwork(String),
    VolumesUpdated(Vec<VolumeEntry>),
    SelectVolume(usize),
    #[allow(dead_code)]
    RemoveVolume(String),
}

#[derive(Clone, Debug)]
pub struct ContainerOpts {
    pub image: String,
    pub cmd: String,
    pub shell: String,
    pub user: String,
    pub workdir: String,
    pub env_vars: String,
    pub port_mapping: String,
    pub volumes: String,
    pub name: String,
    pub autoremove: bool,
}

#[derive(Clone, Debug)]
pub enum Command {
    InspectContainer(String),
    FetchLogs(String),
    StartContainer(String),
    StopContainer(String),
    RestartContainer(String),
    DeleteContainer(String),
    RemoveImage(String),
    CreateContainer(ContainerOpts),
    RemoveNetwork(String),
    RemoveVolume(String),
    SaveConfig,
    CheckUpdate,
    DownloadUpdate { version: String, download_url: String },
    ExportLogs(String, Vec<String>),
}
