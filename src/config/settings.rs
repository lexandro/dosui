//! Application settings (`~/.config/dosui/settings.toml`).
//!
//! Kept intentionally tiny for M1: just where the DOSBox binary lives. Loading
//! is infallible — a missing or broken file yields defaults so the UI always
//! starts — while saving surfaces errors.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::paths;

const SETTINGS_FILE: &str = "settings.toml";

/// User preferences. Grows in later milestones (UI prefs, default games dir, …).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    /// Explicit DOSBox binary path. When unset, the launcher auto-detects it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dosbox_path: Option<PathBuf>,
}

impl AppSettings {
    /// Load settings, falling back to defaults on any error (logged).
    pub fn load() -> AppSettings {
        let path = match paths::config_dir() {
            Ok(dir) => dir.join(SETTINGS_FILE),
            Err(e) => {
                log::warn!("no config dir, using default settings: {e:#}");
                return AppSettings::default();
            }
        };
        match fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text).unwrap_or_else(|e| {
                log::warn!("bad {}, using defaults: {e}", path.display());
                AppSettings::default()
            }),
            Err(_) => AppSettings::default(), // not created yet
        }
    }

    /// Persist settings to `settings.toml`, creating the config dir if needed.
    pub fn save(&self) -> Result<()> {
        let dir = paths::config_dir()?;
        fs::create_dir_all(&dir)
            .with_context(|| format!("creating config dir {}", dir.display()))?;
        let text = toml::to_string_pretty(self).context("serializing settings")?;
        let path = dir.join(SETTINGS_FILE);
        fs::write(&path, text).with_context(|| format!("writing {}", path.display()))
    }
}
