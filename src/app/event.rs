use crate::app::mode::Mode;
use crate::app::state::ExplorerEntry;

#[derive(Clone, Debug)]
pub struct ContainerSummary {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    pub status: String,
    pub ports: String,
    pub project: String,
    pub health: String,
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
    pub timestamp: String,
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
    pub size: i64,
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
    RestartPolicy,
    MemoryLimit,
    CpuLimit,
    Network,
    Labels,
    Privileged,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConfirmAction {
    DeleteContainer(String),
    BatchDeleteContainers,
    RemoveImage(String),
    RemoveDanglingImages,
    PruneUnusedImages,
    RemoveNetwork(String),
    RemoveVolume(String),
    BatchDeleteImages,
    BatchDeleteNetworks,
    BatchDeleteVolumes,
    DeleteHostFile(String),
    DeleteContainerFile(String, String),
}

#[derive(Clone, Debug)]
pub enum MouseClickKind {
    Left,
    ScrollUp,
    ScrollDown,
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

    ContainersRefreshNeeded,
    ContainersUpdated(Vec<ContainerSummary>),
    SelectContainer(usize),
    FilterContainers(String),
    ActivateFilter,
    FilterSubmit(Option<usize>),
    RestartContainer(String),
    StopContainer(String),
    ContainerStopped(String),
    ContainerStarted(String),
    StartContainer(String),
    ContainerDeleted(String),
    ShowDetails,
    Inspected(serde_json::Value, String),
    ScrollDetails(i32),
    ToggleSelectContainer(String),
    SelectAllContainers,
    BatchToggleContainers(Vec<String>),
    CycleStatusFilter,

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
    PrunedImages(usize),
    RunImage(String, String),
    ImageRunFieldUpdate(ImageRunField, String),
    ImageRunToggleAutoremove,
    ImageRunToggleAdvanced,
    ImageRunFocusNext,
    ImageRunFocusPrev,
    ImageRunSubmit,

    CheckUpdate,
    UpdateAvailable(String, String),
    ToggleMouse,
    MouseClick { row: u16, col: u16, kind: MouseClickKind },

     EventsUpdated(Vec<DockerEvent>),
    ActivateEventsFilter,
    EventsFilterSubmit,
    FilterEvents(String),
    ScrollEvents(i32),
    ToggleLogTimestamps,

    StatisticsUpdated(Vec<StatEntry>),
    NetworksUpdated(Vec<NetworkEntry>),
    SelectNetwork(usize),
    VolumesUpdated(Vec<VolumeEntry>),
    SelectVolume(usize),

    ToggleSelectionMode,
    ToggleSelectResource(String),
    SelectAllResources,
    ScrollStatistics(i32),
    CycleSortStat(i32),
    ToggleSortDirection,

    SortByColumn(usize),
    ToggleColumnPicker,
    ToggleColumn(String),

    ExplorerSelect,
    ExplorerCopyToContainer,
    ExplorerCopyFromContainer,
    ExplorerTransferComplete(String),
    ExplorerTransferError(String),
    ExplorerFilter(String),
    ExplorerContainerDirUpdated(String, String, Vec<ExplorerEntry>),
    ExplorerHostGoUp,
    ExplorerContainerGoUp,
    ExplorerHostSelect(usize),
    ExplorerContainerSelect(usize),
    ExplorerHostEnterDir(String),
    ExplorerContainerEnterDir(String),
    ExplorerHostRefresh,
    ExplorerContainerRefresh,
    ExplorerHostActivateFilter,
    ExplorerContainerActivateFilter,
    ExplorerFilterSubmit,
    ExplorerHostActivateRename,
    ExplorerContainerActivateRename,
    ExplorerRenameUpdate(String),
    ExplorerRenameCancel,
    ExplorerRenameSubmit,
    ExplorerHostDirUpdated(String, Vec<ExplorerEntry>),
    ContainerWorkingDir(String, String),

    StartDiagnostics(String),
    DiagnosticsPhaseUpdate(crate::app::state::DiagnosticsPhase),
    DiagnosticsChunk(String),
    DiagnosticsPlaybook(String),
    DiagnosticsDone,
    DiagnosticsError(String),
    ScrollDiagnostics(i32),
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
    pub restart_policy: String,
    pub memory_limit: String,
    pub cpu_limit: String,
    pub network: String,
    pub labels: String,
    pub privileged: bool,
}

#[derive(Clone, Debug)]
pub enum Command {
    RefreshContainers,
    InspectContainer(String),
    FetchLogs(String),
    StartContainer(String),
    StopContainer(String),
    RestartContainer(String),
    DeleteContainer(String),
    RemoveImage(String),
    RemoveDanglingImages,
    PruneUnusedImages,
    CreateContainer(ContainerOpts),
    RemoveNetwork(String),
    RemoveVolume(String),
    ToggleMouseCapture,
    BatchToggleContainers(Vec<String>),
    BatchDeleteContainers(Vec<String>),
    BatchDeleteImages(Vec<String>),
    BatchDeleteNetworks(Vec<String>),
    BatchDeleteVolumes(Vec<String>),
    SaveConfig,
    CheckUpdate,
    DownloadUpdate { version: String, download_url: String },
    ExportLogs(String, Vec<String>),
    ListContainerDir(String, String),
    ListHostDir(String),
    ListVolumeDir(String, String),
    CopyToContainer(String, String, String),
    CopyFromContainer(String, String, String),
    DeleteHostFile(String),
    DeleteContainerFile(String, String),
    RenameHostFile(String, String),
    RenameContainerFile(String, String, String),
    FetchContainerWorkingDir(String),
    RemoveVolumeHelper(String),
    StartDiagnostics(String),
}
