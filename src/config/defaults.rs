//! Global DOSBox defaults (`~/.config/dosui/defaults.toml`).
//!
//! These are the base [`DosboxConfig`] every profile inherits from:
//! `effective = defaults.merge(profile.dosbox)` (see [`DosboxConfig::merge`]).
//! Loading is infallible (missing/broken file -> empty defaults); saving is checked.

use std::fs;

use anyhow::{Context, Result};

use super::dosbox_conf::DosboxConfig;
use super::paths;

const DEFAULTS_FILE: &str = "defaults.toml";

/// Load the global defaults, falling back to empty defaults on any error (logged).
pub fn load() -> DosboxConfig {
    let path = match paths::config_dir() {
        Ok(dir) => dir.join(DEFAULTS_FILE),
        Err(e) => {
            log::warn!("no config dir, using empty defaults: {e:#}");
            return DosboxConfig::default();
        }
    };
    match fs::read_to_string(&path) {
        Ok(text) => toml::from_str(&text).unwrap_or_else(|e| {
            log::warn!("bad {}, using empty defaults: {e}", path.display());
            DosboxConfig::default()
        }),
        Err(_) => DosboxConfig::default(), // not created yet
    }
}

/// Persist the global defaults, creating the config dir if needed.
pub fn save(defaults: &DosboxConfig) -> Result<()> {
    let dir = paths::config_dir()?;
    fs::create_dir_all(&dir).with_context(|| format!("creating config dir {}", dir.display()))?;
    let text = toml::to_string_pretty(defaults).context("serializing defaults")?;
    let path = dir.join(DEFAULTS_FILE);
    fs::write(&path, text).with_context(|| format!("writing {}", path.display()))
}
