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
}

impl Default for VolumeColumns {
    fn default() -> Self {
        Self {
            show_name: true,
            show_driver: true,
            show_mountpoint: true,
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
    #[serde(default = "default_statistics_sort_desc", deserialize_with = "string_or_vec")]
    pub statistics_sort_desc: Vec<String>,
    #[serde(default = "default_logs_export", deserialize_with = "string_or_vec")]
    pub logs_export: Vec<String>,
    #[serde(default = "default_toggle_timestamps", deserialize_with = "string_or_vec")]
    pub toggle_timestamps: Vec<String>,
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
    default_statistics_sort_desc => vec!["t"];
    default_logs_export => vec!["Ctrl+S"];
    default_toggle_timestamps => vec!["T"];
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
            statistics_sort_desc: default_statistics_sort_desc(),
            logs_export: default_logs_export(),
            toggle_timestamps: default_toggle_timestamps(),
        }
    }
}

impl Keybindings {
    pub fn to_help_text(&self) -> Vec<String> {
        vec![
            format!("    {}           Quit", key_first(&self.quit)),
            format!("    {}           Go back / close", key_first(&self.back)),
            "".to_string(),
            "  GLOBAL NAVIGATION".to_string(),
            "    c           Containers view".to_string(),
            "    i           Images view".to_string(),
            "    n           Networks view".to_string(),
            "    v           Volumes view".to_string(),
            "    e           Events view".to_string(),
            "    %           Statistics view".to_string(),
            format!("    {}           Help view", key_first(&self.help)),
            "    Tab         Next tab".to_string(),
            "    S-Tab       Previous tab".to_string(),
            "    U           Check for updates / download".to_string(),
            "".to_string(),
            "  CONTAINERS".to_string(),
            format!("    {} / {}     Navigate down", key_first(&self.navigate_down), "↓"),
            format!("    {} / {}     Navigate up", key_first(&self.navigate_up), "↑"),
            format!("    {}           Open details", key_first(&self.open_details)),
            format!("    {}           Open logs", key_first(&self.open_logs)),
            format!("    {}           Open shell (exec)", key_first(&self.open_shell)),
            "    x           Open file explorer".to_string(),
            format!("    {}           Start/Stop container", key_first(&self.start_stop)),
            format!("    {}           Restart container", key_first(&self.restart)),
            format!("    {}           Delete container", key_first(&self.delete)),
            format!("    {}           Search/filter", key_first(&self.search)),
            format!("    {}           Toggle selection mode", key_first(&self.toggle_selection)),
            format!("    {}           Select all (in selection mode)", key_first(&self.select_all)),
            "    Ctrl+Y      Copy container ID".to_string(),
            "    Ctrl+O      Column picker".to_string(),
            "".to_string(),
            "  CONTAINER DETAILS".to_string(),
            format!("    {}           Open logs", key_first(&self.open_logs)),
            format!("    {}           Open shell (exec)", key_first(&self.open_shell)),
            format!("    {}           Restart container", key_first(&self.restart)),
            format!("    {}           Start/Stop container", key_first(&self.start_stop)),
            "    x           Open file explorer".to_string(),
            format!("    {} / {}     Scroll down", key_first(&self.navigate_up), "↓"),
            format!("    {} / {}     Scroll up", key_first(&self.navigate_down), "↑"),
            "    PgDn        Page down".to_string(),
            "    PgUp        Page up".to_string(),
            format!("    {}           Jump to top", key_first(&self.jump_top)),
            format!("    {}           Jump to bottom", key_first(&self.jump_bottom)),
            "".to_string(),
            "  IMAGES".to_string(),
            format!("    {} / {}     Navigate down", key_first(&self.navigate_images), "↓"),
            format!("    {} / {}     Navigate up", key_first(&self.navigate_up), "↑"),
            format!("    {}           Run image", key_first(&self.run_image)),
            format!("    {}           Remove image", key_first(&self.remove_image)),
            format!("    {}           Remove dangling images", key_first(&self.remove_dangling_images)),
            format!("    {}           Prune unused images", key_first(&self.prune_images)),
            format!("    {}           Search/filter", key_first(&self.search)),
            "    Ctrl+Y      Copy image ID".to_string(),
            "    Ctrl+O      Column picker".to_string(),
            "".to_string(),
            "  IMAGE RUN FORM".to_string(),
            "    Tab / ↓     Next field".to_string(),
            "    ↑           Previous field".to_string(),
            "    a           Toggle autoremove / restart policy / privileged".to_string(),
            "    Ctrl+A      Toggle advanced options".to_string(),
            "    Enter       Create and run container".to_string(),
            "".to_string(),
            "  SHELL CONFIG FORM".to_string(),
            "    Tab / ↓     Next field".to_string(),
            "    ↑           Previous field".to_string(),
            "    Enter       Save config + launch shell".to_string(),
            "".to_string(),
            "  LOGS".to_string(),
            format!("    {} / {}     Scroll down", key_first(&self.navigate_down), "↓"),
            format!("    {} / {}     Scroll up", key_first(&self.navigate_up), "↑"),
            "    PgDn        Page down".to_string(),
            "    PgUp        Page up".to_string(),
            format!("    {}           Jump to bottom", key_first(&self.jump_bottom)),
            format!("    {}           Jump to top", key_first(&self.jump_top)),
            format!("    {}           Search within logs", key_first(&self.search)),
            "    Space / p   Pause/unpause auto-scroll".to_string(),
            "    r           Reconnect (when paused)".to_string(),
            "    s / Ctrl+S  Export logs to file".to_string(),
            format!("    {}           Toggle timestamps", key_first(&self.toggle_timestamps)),
            "".to_string(),
            "  EVENTS".to_string(),
            format!("    {} / {}     Scroll down", key_first(&self.navigate_down), "↓"),
            format!("    {} / {}     Scroll up", key_first(&self.navigate_up), "↑"),
            "    PgDn        Page down".to_string(),
            "    PgUp        Page up".to_string(),
            format!("    {}           Jump to top", key_first(&self.jump_top)),
            format!("    {}           Jump to bottom", key_first(&self.jump_bottom)),
            format!("    {}           Filter events", key_first(&self.search)),
            "".to_string(),
            "  STATISTICS".to_string(),
            "    ← / →      Cycle sort field".to_string(),
            format!("    {}           Toggle sort direction", key_first(&self.statistics_sort_desc)),
            "".to_string(),
            "  NETWORKS".to_string(),
            format!("    {} / {}     Navigate down", key_first(&self.navigate_down), "↓"),
            format!("    {} / {}     Navigate up", key_first(&self.navigate_up), "↑"),
            format!("    {}           Delete selected", key_first(&self.delete)),
            format!("    {}           Search/filter", key_first(&self.search)),
            "    Ctrl+Y      Copy network ID".to_string(),
            "    Ctrl+O      Column picker".to_string(),
            "".to_string(),
            "  VOLUMES".to_string(),
            format!("    {} / {}     Navigate down", key_first(&self.navigate_down), "↓"),
            format!("    {} / {}     Navigate up", key_first(&self.navigate_up), "↑"),
            format!("    {}           Delete selected", key_first(&self.delete)),
            format!("    {}           Search/filter", key_first(&self.search)),
            "    Ctrl+Y      Copy volume name".to_string(),
            "    Ctrl+O      Column picker".to_string(),
            "".to_string(),
            "  EXPLORER".to_string(),
            "    Tab         Switch between host/container panels".to_string(),
            format!("    {} / {}     Navigate entries", key_first(&self.navigate_down), "↓"),
            format!("    {} / {}     Navigate up", key_first(&self.navigate_up), "↑"),
            "    Enter       Open directory".to_string(),
            "    Backspace   Go up to parent".to_string(),
            "    PgDn        Page down".to_string(),
            "    PgUp        Page up".to_string(),
            format!("    {}           Jump to beginning", key_first(&self.jump_top)),
            format!("    {}           Jump to end", key_first(&self.jump_bottom)),
            "    Ctrl+C      Copy file between panels".to_string(),
            "    d           Delete file/dir".to_string(),
            "    r           Rename file/dir".to_string(),
            "    R           Refresh panel".to_string(),
            format!("    {}           Filter entries", key_first(&self.search)),
            "".to_string(),
            format!("  Press {} or {} to close", key_first(&self.back), key_first(&self.help)),
        ]
    }
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
        }
    }
}

impl OmdockerConfig {
    fn path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config").join("omdocker").join("omdocker.toml")
    }

    pub fn load() -> Self {
        let path = Self::path();
        let config: Self = std::fs::read_to_string(&path)
            .ok()
            .and_then(|content| toml::from_str(&content).ok())
            .unwrap_or_default();
        let _ = config.save();
        config
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
