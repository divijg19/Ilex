use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::cli::OutputMode;
use crate::modules::ModuleEntry;

const KNOWN_MODULE_KEYS: [&str; 6] = ["os", "cpu", "memory", "disk", "shell", "terminal"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigState {
    description: String,
    default_output_mode: Option<OutputMode>,
    module_order: Vec<String>,
    module_enabled: BTreeMap<String, bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    Io { path: PathBuf, message: String },
    Parse { path: PathBuf, message: String },
    Validation(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, message } => {
                write!(
                    formatter,
                    "failed to read config {}: {message}",
                    path.display()
                )
            }
            Self::Parse { path, message } => {
                write!(formatter, "invalid config {}: {message}", path.display())
            }
            Self::Validation(message) => formatter.write_str(message),
        }
    }
}

impl ConfigState {
    pub fn load() -> Result<Self, ConfigError> {
        match default_config_path() {
            Some(path) => Self::load_from_path_if_exists(&path),
            None => Ok(Self::defaults(
                "defaults (home directory unavailable)".to_owned(),
            )),
        }
    }

    pub fn bootstrap_defaults() -> Self {
        Self::defaults("defaults (no config file found)".to_owned())
    }

    pub fn load_from_path(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path).map_err(|error| ConfigError::Io {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
        let raw: RawConfig = toml::from_str(&content).map_err(|error| ConfigError::Parse {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;

        Self::from_raw(raw, format!("config loaded from {}", path.display()))
    }

    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    pub fn resolve_output_mode(&self, requested: Option<OutputMode>) -> OutputMode {
        requested
            .or(self.default_output_mode)
            .unwrap_or(OutputMode::Fetch)
    }

    pub fn apply_module_preferences(&self, entries: Vec<ModuleEntry>) -> Vec<ModuleEntry> {
        let mut filtered: Vec<ModuleEntry> = entries
            .into_iter()
            .filter(|entry| self.module_enabled.get(entry.key).copied().unwrap_or(true))
            .collect();

        if self.module_order.is_empty() {
            return filtered;
        }

        let mut ordered = Vec::with_capacity(filtered.len());
        for key in &self.module_order {
            if let Some(index) = filtered.iter().position(|entry| entry.key == key.as_str()) {
                ordered.push(filtered.remove(index));
            }
        }
        ordered.extend(filtered);
        ordered
    }

    fn load_from_path_if_exists(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Ok(Self::defaults(format!(
                "defaults (no config at {})",
                path.display()
            )));
        }

        Self::load_from_path(path)
    }

    fn defaults(description: String) -> Self {
        Self {
            description,
            default_output_mode: None,
            module_order: Vec::new(),
            module_enabled: BTreeMap::new(),
        }
    }

    fn from_raw(raw: RawConfig, description: String) -> Result<Self, ConfigError> {
        let default_output_mode = raw
            .output
            .and_then(|output| output.default_mode)
            .map(|mode| {
                OutputMode::from_config_value(&mode).ok_or_else(|| {
                    ConfigError::Validation(format!(
                        "invalid output.default_mode `{mode}`; expected fetch, minimal, or json"
                    ))
                })
            })
            .transpose()?;

        let mut module_order = Vec::new();
        let mut module_enabled = BTreeMap::new();

        if let Some(modules) = raw.modules {
            if let Some(order) = modules.order {
                let mut seen = BTreeSet::new();
                for key in order {
                    validate_module_key(&key)?;
                    if !seen.insert(key.clone()) {
                        return Err(ConfigError::Validation(format!(
                            "duplicate module key `{key}` in modules.order"
                        )));
                    }
                    module_order.push(key);
                }
            }

            if let Some(enabled) = modules.enabled {
                for (key, value) in enabled {
                    validate_module_key(&key)?;
                    module_enabled.insert(key, value);
                }
            }
        }

        Ok(Self {
            description,
            default_output_mode,
            module_order,
            module_enabled,
        })
    }
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    output: Option<RawOutput>,
    modules: Option<RawModules>,
}

#[derive(Debug, Deserialize)]
struct RawOutput {
    default_mode: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawModules {
    order: Option<Vec<String>>,
    enabled: Option<BTreeMap<String, bool>>,
}

fn default_config_path() -> Option<PathBuf> {
    env::var_os("HOME").map(|home| {
        PathBuf::from(home)
            .join(".config")
            .join("corefetch")
            .join("config.toml")
    })
}

fn validate_module_key(key: &str) -> Result<(), ConfigError> {
    if KNOWN_MODULE_KEYS.contains(&key) {
        Ok(())
    } else {
        Err(ConfigError::Validation(format!(
            "unknown module key `{key}` in config"
        )))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{ConfigError, ConfigState};
    use crate::cli::OutputMode;
    use crate::modules::ModuleEntry;

    #[test]
    fn loads_config_with_output_default_and_module_preferences() {
        let config = ConfigState::load_from_path(&config_fixture_path("basic.toml"))
            .expect("config should load");
        let entries = vec![
            ModuleEntry {
                key: "os",
                label: "OS",
                value: "Fedora Linux 43".to_owned(),
            },
            ModuleEntry {
                key: "cpu",
                label: "CPU",
                value: "ExampleCore 9000 (4 cores)".to_owned(),
            },
            ModuleEntry {
                key: "memory",
                label: "Memory",
                value: "7.8 GiB / 31.2 GiB".to_owned(),
            },
            ModuleEntry {
                key: "disk",
                label: "Disk",
                value: "31.2 GiB / 62.5 GiB (/)".to_owned(),
            },
            ModuleEntry {
                key: "shell",
                label: "Shell",
                value: "fish".to_owned(),
            },
            ModuleEntry {
                key: "terminal",
                label: "Terminal",
                value: "vscode (xterm-256color)".to_owned(),
            },
        ];

        let filtered = config.apply_module_preferences(entries);

        assert_eq!(config.resolve_output_mode(None), OutputMode::Minimal);
        assert_eq!(filtered[0].key, "shell");
        assert_eq!(filtered[1].key, "os");
        assert!(!filtered.iter().any(|entry| entry.key == "terminal"));
        assert!(config.description().contains("basic.toml"));
    }

    #[test]
    fn uses_requested_output_mode_over_config_default() {
        let config = ConfigState::load_from_path(&config_fixture_path("basic.toml"))
            .expect("config should load");

        assert_eq!(
            config.resolve_output_mode(Some(OutputMode::Json)),
            OutputMode::Json
        );
    }

    #[test]
    fn missing_config_path_falls_back_to_defaults() {
        let config = ConfigState::load_from_path_if_exists(&config_fixture_path("missing.toml"))
            .expect("missing config should fall back to defaults");

        assert_eq!(config.resolve_output_mode(None), OutputMode::Fetch);
        assert!(config.description().contains("defaults"));
    }

    #[test]
    fn rejects_invalid_output_mode() {
        let error = ConfigState::load_from_path(&config_fixture_path("invalid-mode.toml"))
            .expect_err("invalid mode should fail");

        assert!(matches!(error, ConfigError::Validation(_)));
        assert!(error.to_string().contains("output.default_mode"));
    }

    #[test]
    fn rejects_unknown_module_key() {
        let error = ConfigState::load_from_path(&config_fixture_path("unknown-module.toml"))
            .expect_err("unknown module key should fail");

        assert!(matches!(error, ConfigError::Validation(_)));
        assert!(error.to_string().contains("unknown module key"));
    }

    fn config_fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("config")
            .join(name)
    }
}
