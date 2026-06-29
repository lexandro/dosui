//! XDG path resolution for dosui.
//!
//! - config  (`~/.config/dosui`)      : user preferences + global defaults
//! - data    (`~/.local/share/dosui`) : the profile library (the valuable content)

use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;

use crate::app::APP_NAME;

/// Resolve the platform `ProjectDirs` for dosui.
fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("io.github", "dosui", APP_NAME)
        .context("could not determine XDG base directories for dosui")
}

/// `~/.config/dosui` — settings.toml, defaults.toml.
pub fn config_dir() -> Result<PathBuf> {
    Ok(project_dirs()?.config_dir().to_path_buf())
}

/// `~/.local/share/dosui` — the data root.
pub fn data_dir() -> Result<PathBuf> {
    Ok(project_dirs()?.data_dir().to_path_buf())
}

/// `~/.local/share/dosui/profiles` — one subdirectory per game profile.
pub fn profiles_dir() -> Result<PathBuf> {
    Ok(data_dir()?.join("profiles"))
}
