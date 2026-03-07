use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Top-level application configuration, loaded from `~/.config/kuberift/config.toml`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub ui: UiConfig,
}

/// `[general]` section — defaults for context, namespace, editor, shell, read-only mode.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct GeneralConfig {
    #[serde(default)]
    pub default_namespace: String,
    #[serde(default)]
    pub default_context: String,
    #[serde(default)]
    pub default_resource: String,
    #[serde(default)]
    pub editor: String,
    #[serde(default)]
    pub shell: String,
    #[serde(default)]
    pub read_only: bool,
}

/// `[ui]` section — display preferences.
#[derive(Debug, Clone, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_true")]
    pub show_namespace: bool,
    #[serde(default = "default_true")]
    pub show_age: bool,
    #[serde(default)]
    pub show_context: bool,
    #[serde(default = "default_truncate_length")]
    pub truncate_name_length: usize,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_namespace: true,
            show_age: true,
            show_context: false,
            truncate_name_length: default_truncate_length(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_truncate_length() -> usize {
    48
}

/// Returns the path to the config file: `$XDG_CONFIG_HOME/kuberift/config.toml`.
pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("kuberift").join("config.toml"))
}

/// Load config from the default XDG path. Returns defaults if file is missing.
pub fn load_config() -> Config {
    config_path()
        .as_deref()
        .and_then(load_from_path)
        .unwrap_or_default()
}

/// Load and parse config from a specific path. Returns `None` if file doesn't exist.
pub fn load_from_path(path: &Path) -> Option<Config> {
    let raw = std::fs::read_to_string(path).ok()?;
    Some(parse_config(&raw, path))
}

/// Parse a TOML string into `Config`. Warns on parse errors, returns defaults.
pub fn parse_config(raw: &str, source: &Path) -> Config {
    toml::from_str(raw).unwrap_or_else(|err| {
        eprintln!(
            "[kuberift] warning: failed to parse config '{}': {err}",
            source.display()
        );
        Config::default()
    })
}
