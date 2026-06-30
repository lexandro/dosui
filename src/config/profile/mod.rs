//! Game profile model — the serializable content of each `profile.toml`.
//!
//! A profile is one DOS game/program: its metadata plus a [`RunSpec`] describing
//! what to mount and run, plus per-profile DOSBox `dosbox` overrides. Profiles
//! live one-per-directory under [`crate::config::paths::profiles_dir`]; the
//! directory name is the profile id.
//!
//! This module holds the data types; loading/scanning lives in [`store`], id/dir
//! naming in [`naming`], and small time helpers in [`util`].

mod naming;
mod store;
mod util;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::dosbox_conf::DosboxConfig;

pub use naming::{duplicate, new_profile_dir};
pub use store::{scan, scan_executables};
pub use util::{humanize_since, now_unix};

/// File name of the profile descriptor inside each profile directory.
pub const PROFILE_FILE: &str = "profile.toml";

/// One game/program managed by dosui. Round-trips to `profile.toml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    /// Stable id; also the on-disk directory name.
    pub id: String,
    /// Display name, e.g. "Dune 2".
    pub title: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub year: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub developer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub www: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Cover image path, relative to the profile directory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub favorite: bool,
    /// Unix time (seconds) of the last launch; `None` until first played.
    /// Declared before the table fields so it serializes as a top-level TOML key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_played: Option<u64>,

    /// What to mount and execute. Drives the generated `[autoexec]`.
    pub run: RunSpec,

    /// Per-profile DOSBox settings. Unset (`None`) leaves inherit from the
    /// global defaults (see [`DosboxConfig::merge`]).
    #[serde(default)]
    pub dosbox: DosboxConfig,
}

/// What DOSBox should mount and run for this profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunSpec {
    #[serde(default)]
    pub mounts: Vec<Mount>,
    /// Drive switched to before running the command, e.g. 'C'.
    pub working_drive: char,
    /// Program to run, e.g. "DUNE2.EXE" or "INSTALL.BAT".
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    /// Append `exit` to autoexec so DOSBox closes when the program quits.
    #[serde(default)]
    pub exit_after: bool,
}

/// A single DOSBox drive mount (a directory or a disk image).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mount {
    /// Drive letter, e.g. 'C'.
    pub drive: char,
    pub kind: MountKind,
    /// Host path to the directory or image file.
    pub path: PathBuf,
    /// Optional volume label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// How a [`Mount`] is attached. Maps to `mount` vs `imgmount -t …`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MountKind {
    /// A host directory mounted as a drive (`mount C /path`).
    Directory,
    /// CD image (`imgmount D image.cue -t cdrom`).
    CdImage,
    /// Floppy image (`imgmount A image.img -t floppy`).
    FloppyImage,
    /// Hard-disk image (`imgmount C image.img -t hdd`).
    HddImage,
}

fn is_false(b: &bool) -> bool {
    !*b
}
