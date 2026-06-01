use std::collections::HashMap;
use std::path::PathBuf;
use crate::input::keymap::KeyMap;

fn string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;
    struct StringOrVec;
    impl<'de> de::Visitor<'de> for StringOrVec {
        type Value = Vec<String>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string or list of strings")
        }
        fn visit_str<E: de::Error>(self, value: &str) -> Result<Vec<String>, E> {
            Ok(vec![value.to_string()])
        }
        fn visit_seq<S: de::SeqAccess<'de>>(self, mut seq: S) -> Result<Vec<String>, S::Error> {
            let mut vec = Vec::new();
            while let Some(s) = seq.next_element::<String>()? {
                vec.push(s);
            }
            Ok(vec)
        }
    }
    deserializer.deserialize_any(StringOrVec)
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct ContainerShellConfig {
    pub shell: Option<String>,
    pub user: Option<String>,
    pub workdir: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PollingIntervals {
    pub containers_ms: u64,
    pub statistics_ms: u64,
    pub images_ms: u64,
    pub networks_ms: u64,
    pub volumes_ms: u64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ContainerColumns {
    pub show_name: bool,
    pub show_image: bool,
    pub show_state: bool,
    pub show_ports: bool,
}

impl Default for ContainerColumns {
    fn default() -> Self {
        Self {
            show_name: true,
            show_image: true,
            show_state: true,
            show_ports: true,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ImageColumns {
    pub show_repository: bool,
    pub show_tag: bool,
    pub show_id: bool,
    pub show_size: bool,
}

impl Default for ImageColumns {
    fn default() -> Self {
        Self {
            show_repository: true,
            show_tag: true,
            show_id: true,
            show_size: true,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NetworkColumns {
    pub show_name: bool,
    pub show_id: bool,
    pub show_driver: bool,
    pub show_scope: bool,
    pub show_ipam: bool,
}

impl Default for NetworkColumns {
    fn default() -> Self {
        Self {
            show_name: true,
            show_id: true,
            show_driver: true,
            show_scope: true,
            show_ipam: true,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct VolumeColumns {
    pub show_name: bool,
    pub show_driver: bool,
    pub show_mountpoint: bool,
    pub show_size: bool,
}

impl Default for VolumeColumns {
    fn default() -> Self {
        Self {
            show_name: true,
            show_driver: true,
            show_mountpoint: true,
            show_size: true,
        }
    }
}

impl Default for PollingIntervals {
    fn default() -> Self {
        Self {
            containers_ms: 2000,
            statistics_ms: 2000,
            images_ms: 10000,
            networks_ms: 10000,
            volumes_ms: 10000,
        }
    }
}

impl PollingIntervals {
    pub fn clamp(&mut self) {
        const MIN_MS: u64 = 500;
        const MAX_MS: u64 = 60000;
        self.containers_ms = self.containers_ms.clamp(MIN_MS, MAX_MS);
        self.statistics_ms = self.statistics_ms.clamp(MIN_MS, MAX_MS);
        self.images_ms = self.images_ms.clamp(MIN_MS, MAX_MS);
        self.networks_ms = self.networks_ms.clamp(MIN_MS, MAX_MS);
        self.volumes_ms = self.volumes_ms.clamp(MIN_MS, MAX_MS);
    }
}

fn key_first(keys: &[String]) -> String {
    keys.first().cloned().unwrap_or_default()
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Keybindings {
    #[serde(default = "default_quit", deserialize_with = "string_or_vec")]
    pub quit: Vec<String>,
    #[serde(default = "default_back", deserialize_with = "string_or_vec")]
    pub back: Vec<String>,
    #[serde(default = "default_help", deserialize_with = "string_or_vec")]
    pub help: Vec<String>,
    #[serde(default = "default_navigate_down", deserialize_with = "string_or_vec")]
    pub navigate_down: Vec<String>,
    #[serde(default = "default_navigate_up", deserialize_with = "string_or_vec")]
    pub navigate_up: Vec<String>,
    #[serde(default = "default_open_details", deserialize_with = "string_or_vec")]
    pub open_details: Vec<String>,
    #[serde(default = "default_open_logs", deserialize_with = "string_or_vec")]
    pub open_logs: Vec<String>,
    #[serde(default = "default_open_shell", deserialize_with = "string_or_vec")]
    pub open_shell: Vec<String>,
    #[serde(default = "default_start_stop", deserialize_with = "string_or_vec")]
    pub start_stop: Vec<String>,
    #[serde(default = "default_restart", deserialize_with = "string_or_vec")]
    pub restart: Vec<String>,
    #[serde(default = "default_delete", deserialize_with = "string_or_vec")]
    pub delete: Vec<String>,
    #[serde(default = "default_search", deserialize_with = "string_or_vec")]
    pub search: Vec<String>,
    #[serde(default = "default_toggle_selection", deserialize_with = "string_or_vec")]
    pub toggle_selection: Vec<String>,
    #[serde(default = "default_select_all", deserialize_with = "string_or_vec")]
    pub select_all: Vec<String>,
    #[serde(default = "default_navigate_images", deserialize_with = "string_or_vec")]
    pub navigate_images: Vec<String>,
    #[serde(default = "default_run_image", deserialize_with = "string_or_vec")]
    pub run_image: Vec<String>,
    #[serde(default = "default_remove_image", deserialize_with = "string_or_vec")]
    pub remove_image: Vec<String>,
    #[serde(default = "default_remove_dangling_images", deserialize_with = "string_or_vec")]
    pub remove_dangling_images: Vec<String>,
    #[serde(default = "default_prune_images", deserialize_with = "string_or_vec")]
    pub prune_images: Vec<String>,
    #[serde(default = "default_jump_top", deserialize_with = "string_or_vec")]
    pub jump_top: Vec<String>,
    #[serde(default = "default_jump_bottom", deserialize_with = "string_or_vec")]
    pub jump_bottom: Vec<String>,
    #[serde(default = "default_sort_direction", deserialize_with = "string_or_vec")]
    pub sort_direction: Vec<String>,
    #[serde(default = "default_logs_export", deserialize_with = "string_or_vec")]
    pub logs_export: Vec<String>,
    #[serde(default = "default_toggle_timestamps", deserialize_with = "string_or_vec")]
    pub toggle_timestamps: Vec<String>,
    #[serde(default = "default_open_diagnostics", deserialize_with = "string_or_vec")]
    pub open_diagnostics: Vec<String>,
}

macro_rules! default_keys {
    ($($name:ident => $default:expr);* $(;)?) => {
        $(
            fn $name() -> Vec<String> { $default.into_iter().map(|s| s.to_string()).collect() }
        )*
    };
}

default_keys! {
    default_quit => vec!["q"];
    default_back => vec!["Esc"];
    default_help => vec!["?"];
    default_navigate_down => vec!["j", "Down"];
    default_navigate_up => vec!["k", "Up"];
    default_open_details => vec!["Enter"];
    default_open_logs => vec!["l"];
    default_open_shell => vec!["s"];
    default_start_stop => vec!["t"];
    default_restart => vec!["r"];
    default_delete => vec!["d"];
    default_search => vec!["/"];
    default_toggle_selection => vec![" "];
    default_select_all => vec!["Ctrl+A"];
    default_navigate_images => vec!["j", "Down"];
    default_run_image => vec!["r"];
    default_remove_image => vec!["d"];
    default_remove_dangling_images => vec!["D"];
    default_prune_images => vec!["p"];
    default_jump_top => vec!["g"];
    default_jump_bottom => vec!["G"];
    default_logs_export => vec!["Ctrl+S"];
    default_toggle_timestamps => vec!["T"];
    default_sort_direction => vec!["Ctrl+T"];
    default_open_diagnostics => vec!["D"];
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            quit: default_quit(),
            back: default_back(),
            help: default_help(),
            navigate_down: default_navigate_down(),
            navigate_up: default_navigate_up(),
            open_details: default_open_details(),
            open_logs: default_open_logs(),
            open_shell: default_open_shell(),
            start_stop: default_start_stop(),
            restart: default_restart(),
            delete: default_delete(),
            search: default_search(),
            toggle_selection: default_toggle_selection(),
            select_all: default_select_all(),
            navigate_images: default_navigate_images(),
            run_image: default_run_image(),
            remove_image: default_remove_image(),
            remove_dangling_images: default_remove_dangling_images(),
            prune_images: default_prune_images(),
            jump_top: default_jump_top(),
            jump_bottom: default_jump_bottom(),
            sort_direction: default_sort_direction(),
            logs_export: default_logs_export(),
            toggle_timestamps: default_toggle_timestamps(),
            open_diagnostics: default_open_diagnostics(),
        }
    }
}

impl Keybindings {
    pub fn to_help_text(&self) -> Vec<String> {
        #[derive(Clone)]
        enum HelpEntry {
            Section(&'static str),
            Blank,
            Key(String, String),
            Text(String),
        }

        let entries: Vec<HelpEntry> = vec![
            HelpEntry::Section(""),
            HelpEntry::Key(format!("    {} ", key_first(&self.quit)), "Quit".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.back)), "Go back / close".to_string()),
            HelpEntry::Blank,
            HelpEntry::Section("GLOBAL NAVIGATION"),
            HelpEntry::Key("    c ".to_string(), "Containers view".to_string()),
            HelpEntry::Key("    i ".to_string(), "Images view".to_string()),
            HelpEntry::Key("    n ".to_string(), "Networks view".to_string()),
            HelpEntry::Key("    v ".to_string(), "Volumes view".to_string()),
            HelpEntry::Key("    e ".to_string(), "Events view".to_string()),
            HelpEntry::Key("    % ".to_string(), "Statistics view".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.help)), "Help view".to_string()),
            HelpEntry::Key("    Tab     ".to_string(), "Next tab".to_string()),
            HelpEntry::Key("    S-Tab   ".to_string(), "Previous tab".to_string()),
            HelpEntry::Key("    U       ".to_string(), "Check for updates / download".to_string()),
            HelpEntry::Blank,
            HelpEntry::Section("CONTAINERS"),
            HelpEntry::Key(format!("    {} / {} ", key_first(&self.navigate_down), "↓"), "Navigate down".to_string()),
            HelpEntry::Key(format!("    {} / {} ", key_first(&self.navigate_up), "↑"), "Navigate up".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.open_details)), "Open details".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.open_logs)), "Open logs".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.open_shell)), "Open shell (exec)".to_string()),
            HelpEntry::Key("    x       ".to_string(), "Open file explorer".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.start_stop)), "Start/Stop container".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.restart)), "Restart container".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.delete)), "Delete container".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.search)), "Search/filter".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.toggle_selection)), "Toggle selection mode".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.select_all)), "Select all (in selection mode)".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.open_diagnostics)), "AI diagnostics".to_string()),
            HelpEntry::Key("    S       ".to_string(), "Cycle status filter (All/Running/Stopped/Paused)".to_string()),
            HelpEntry::Key("    Ctrl+O  ".to_string(), "Column picker".to_string()),
            HelpEntry::Blank,
            HelpEntry::Section("CONTAINER DETAILS"),
            HelpEntry::Key(format!("    {} ", key_first(&self.open_logs)), "Open logs".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.open_shell)), "Open shell (exec)".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.open_diagnostics)), "AI diagnostics".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.restart)), "Restart container".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.start_stop)), "Start/Stop container".to_string()),
            HelpEntry::Key("    x       ".to_string(), "Open file explorer".to_string()),
            HelpEntry::Key(format!("    {} / {} ", key_first(&self.navigate_up), "↓"), "Scroll down".to_string()),
            HelpEntry::Key(format!("    {} / {} ", key_first(&self.navigate_down), "↑"), "Scroll up".to_string()),
            HelpEntry::Key("    PgDn    ".to_string(), "Page down".to_string()),
            HelpEntry::Key("    PgUp    ".to_string(), "Page up".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.jump_top)), "Jump to top".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.jump_bottom)), "Jump to bottom".to_string()),
            HelpEntry::Blank,
            HelpEntry::Section("IMAGES"),
            HelpEntry::Key(format!("    {} / {} ", key_first(&self.navigate_images), "↓"), "Navigate down".to_string()),
            HelpEntry::Key(format!("    {} / {} ", key_first(&self.navigate_up), "↑"), "Navigate up".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.run_image)), "Run image".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.remove_image)), "Remove image".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.remove_dangling_images)), "Remove dangling images".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.prune_images)), "Prune unused images".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.search)), "Search/filter".to_string()),
            HelpEntry::Key("    Ctrl+O  ".to_string(), "Column picker".to_string()),
            HelpEntry::Blank,
            HelpEntry::Section("IMAGE RUN FORM"),
            HelpEntry::Key("    Tab / ↓ ".to_string(), "Next field".to_string()),
            HelpEntry::Key("    ↑         ".to_string(), "Previous field".to_string()),
            HelpEntry::Key("    a         ".to_string(), "Toggle autoremove / restart policy / privileged".to_string()),
            HelpEntry::Key("    Ctrl+A    ".to_string(), "Toggle advanced options".to_string()),
            HelpEntry::Key("    Enter     ".to_string(), "Create and run container".to_string()),
            HelpEntry::Blank,
            HelpEntry::Section("SHELL CONFIG FORM"),
            HelpEntry::Key("    Tab / ↓ ".to_string(), "Next field".to_string()),
            HelpEntry::Key("    ↑         ".to_string(), "Previous field".to_string()),
            HelpEntry::Key("    Enter     ".to_string(), "Save config + launch shell".to_string()),
            HelpEntry::Blank,
            HelpEntry::Section("LOGS"),
            HelpEntry::Key(format!("    {} / {} ", key_first(&self.navigate_down), "↓"), "Scroll down".to_string()),
            HelpEntry::Key(format!("    {} / {} ", key_first(&self.navigate_up), "↑"), "Scroll up".to_string()),
            HelpEntry::Key("    PgDn    ".to_string(), "Page down".to_string()),
            HelpEntry::Key("    PgUp    ".to_string(), "Page up".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.jump_bottom)), "Jump to bottom".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.jump_top)), "Jump to top".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.search)), "Search within logs".to_string()),
            HelpEntry::Key("    Space / p ".to_string(), "Pause/unpause auto-scroll".to_string()),
            HelpEntry::Key("    r         ".to_string(), "Reconnect (when paused)".to_string()),
            HelpEntry::Key("    s / Ctrl+S".to_string(), "Export logs to file".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.toggle_timestamps)), "Toggle timestamps".to_string()),
            HelpEntry::Blank,
            HelpEntry::Section("EVENTS"),
            HelpEntry::Key(format!("    {} / {} ", key_first(&self.navigate_down), "↓"), "Scroll down".to_string()),
            HelpEntry::Key(format!("    {} / {} ", key_first(&self.navigate_up), "↑"), "Scroll up".to_string()),
            HelpEntry::Key("    PgDn    ".to_string(), "Page down".to_string()),
            HelpEntry::Key("    PgUp    ".to_string(), "Page up".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.jump_top)), "Jump to top".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.jump_bottom)), "Jump to bottom".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.search)), "Filter events".to_string()),
            HelpEntry::Blank,
            HelpEntry::Section("STATISTICS"),
            HelpEntry::Key("    ← / →   ".to_string(), "Cycle sort field".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.sort_direction)), "Toggle sort direction".to_string()),
            HelpEntry::Blank,
            HelpEntry::Section("NETWORKS"),
            HelpEntry::Key(format!("    {} / {} ", key_first(&self.navigate_down), "↓"), "Navigate down".to_string()),
            HelpEntry::Key(format!("    {} / {} ", key_first(&self.navigate_up), "↑"), "Navigate up".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.delete)), "Delete selected".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.search)), "Search/filter".to_string()),
            HelpEntry::Key("    Ctrl+O  ".to_string(), "Column picker".to_string()),
            HelpEntry::Blank,
            HelpEntry::Section("VOLUMES"),
            HelpEntry::Key(format!("    {} / {} ", key_first(&self.navigate_down), "↓"), "Navigate down".to_string()),
            HelpEntry::Key(format!("    {} / {} ", key_first(&self.navigate_up), "↑"), "Navigate up".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.delete)), "Delete selected".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.search)), "Search/filter".to_string()),
            HelpEntry::Key("    Ctrl+O  ".to_string(), "Column picker".to_string()),
            HelpEntry::Blank,
            HelpEntry::Section("EXPLORER"),
            HelpEntry::Key("    Tab           ".to_string(), "Switch between host/container panels".to_string()),
            HelpEntry::Key(format!("    {} / {} ", key_first(&self.navigate_down), "↓"), "Navigate entries".to_string()),
            HelpEntry::Key(format!("    {} / {} ", key_first(&self.navigate_up), "↑"), "Navigate up".to_string()),
            HelpEntry::Key("    Enter         ".to_string(), "Open directory".to_string()),
            HelpEntry::Key("    Backspace     ".to_string(), "Go up to parent".to_string()),
            HelpEntry::Key("    PgDn          ".to_string(), "Page down".to_string()),
            HelpEntry::Key("    PgUp          ".to_string(), "Page up".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.jump_top)), "Jump to beginning".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.jump_bottom)), "Jump to end".to_string()),
            HelpEntry::Key("    Ctrl+C        ".to_string(), "Copy file between panels".to_string()),
            HelpEntry::Key("    d             ".to_string(), "Delete file/dir".to_string()),
            HelpEntry::Key("    r             ".to_string(), "Rename file/dir".to_string()),
            HelpEntry::Key("    R             ".to_string(), "Refresh panel".to_string()),
            HelpEntry::Key(format!("    {} ", key_first(&self.search)), "Filter entries".to_string()),
            HelpEntry::Blank,
            HelpEntry::Text(format!("  Press {} or {} to close", key_first(&self.back), key_first(&self.help))),
        ];

        let max_key_width = entries.iter()
            .filter_map(|e| match e {
                HelpEntry::Key(k, _) => Some(k.len()),
                _ => None,
            })
            .max()
            .unwrap_or(0);

        entries.into_iter().map(|e| match e {
            HelpEntry::Section(title) => format!("  {}", title),
            HelpEntry::Blank => String::new(),
            HelpEntry::Key(key, desc) => {
                let padded_key = format!("{:<width$}", key, width = max_key_width);
                format!("{}{}", padded_key, desc)
            }
            HelpEntry::Text(text) => text,
        }).collect()
    }
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LlmConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default)]
    pub api_key: String,
}

fn default_provider() -> String {
    "ollama".to_string()
}

fn default_model() -> String {
    "llama3".to_string()
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OmdockerConfig {
    pub latest_shell: Option<String>,
    #[serde(default)]
    pub containers: HashMap<String, ContainerShellConfig>,
    #[serde(default)]
    pub images: HashMap<String, ContainerShellConfig>,
    #[serde(default)]
    pub check_updates: Option<bool>,
    #[serde(default)]
    pub keybindings: Keybindings,
    #[serde(default)]
    pub polling: PollingIntervals,
    #[serde(default)]
    pub container_columns: ContainerColumns,
    #[serde(default)]
    pub image_columns: ImageColumns,
    #[serde(default)]
    pub network_columns: NetworkColumns,
    #[serde(default)]
    pub volume_columns: VolumeColumns,
    #[serde(default = "default_true")]
    pub mouse: bool,
    #[serde(default)]
    pub llm: Option<LlmConfig>,
}

impl OmdockerConfig {
    pub fn keymap(&self) -> KeyMap {
        KeyMap::from_keybindings(&self.keybindings)
    }
}

impl Default for OmdockerConfig {
    fn default() -> Self {
        Self {
            latest_shell: Some("bash".to_string()),
            containers: HashMap::new(),
            images: HashMap::new(),
            check_updates: None,
            keybindings: Keybindings::default(),
            polling: PollingIntervals::default(),
            container_columns: ContainerColumns::default(),
            image_columns: ImageColumns::default(),
            network_columns: NetworkColumns::default(),
            volume_columns: VolumeColumns::default(),
            mouse: true,
            llm: None,
        }
    }
}

impl OmdockerConfig {
    pub fn path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config").join("omdocker").join("omdocker.toml")
    }

    /// Loads config from disk. Returns (config, parse_error) — `parse_error` is Some
    /// when the file exists but could not be parsed (possibly stale/invalid TOML).
    pub fn load() -> (Self, Option<String>) {
        let path = Self::path();
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return (Self::default(), None),
        };
        match toml::from_str::<Self>(&content) {
            Ok(config) => (config, None),
            Err(e) => (Self::default(), Some(format!("Config parse error: {}", e))),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
        let content = toml::to_string(self).map_err(|e| format!("Failed to serialize config: {}", e))?;
        std::fs::write(&path, content).map_err(|e| format!("Failed to write config: {}", e))?;
        Ok(())
    }
}
