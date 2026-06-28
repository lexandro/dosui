//! Game profile model — the serializable content of each `profile.toml`.
//!
//! A profile is one DOS game/program: its metadata plus a [`RunSpec`] describing
//! what to mount and run. The per-profile DOSBox config *overrides* are attached
//! in M1.2 (see `dosbox_conf`). Profiles live one-per-directory under
//! [`crate::config::paths::profiles_dir`]; the directory name is the profile id.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::dosbox_conf::DosboxConfig;

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

impl Profile {
    /// Load `<profile_dir>/profile.toml`.
    pub fn load(profile_dir: &Path) -> Result<Profile> {
        let path = profile_dir.join(PROFILE_FILE);
        let text = fs::read_to_string(&path)
            .with_context(|| format!("reading profile {}", path.display()))?;
        toml::from_str(&text).with_context(|| format!("parsing profile {}", path.display()))
    }

    /// Write `<profile_dir>/profile.toml`, creating the directory if needed.
    pub fn save(&self, profile_dir: &Path) -> Result<()> {
        fs::create_dir_all(profile_dir)
            .with_context(|| format!("creating profile dir {}", profile_dir.display()))?;
        let text = toml::to_string_pretty(self).context("serializing profile")?;
        let path = profile_dir.join(PROFILE_FILE);
        fs::write(&path, text).with_context(|| format!("writing profile {}", path.display()))
    }
}

/// Scan a profiles root, returning each profile paired with its directory.
///
/// Subdirectories without a readable `profile.toml` are skipped with a warning,
/// so one broken profile never hides the rest. A missing root yields an empty list.
pub fn scan(profiles_dir: &Path) -> Result<Vec<(PathBuf, Profile)>> {
    if !profiles_dir.exists() {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    for entry in fs::read_dir(profiles_dir)
        .with_context(|| format!("reading profiles dir {}", profiles_dir.display()))?
    {
        let dir = entry?.path();
        if !dir.is_dir() {
            continue;
        }
        match Profile::load(&dir) {
            Ok(profile) => out.push((dir, profile)),
            Err(e) => log::warn!("skipping {}: {e:#}", dir.display()),
        }
    }
    out.sort_by_key(|(_, p)| p.title.to_lowercase());
    Ok(out)
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// Current Unix time in seconds (0 if the clock is before the epoch).
pub fn now_unix() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// A coarse human-readable "time ago" string for a past Unix timestamp.
pub fn humanize_since(now: u64, then: u64) -> String {
    if then > now {
        return "just now".to_string();
    }
    let secs = now - then;
    match secs {
        0..=59 => "just now".to_string(),
        60..=3599 => format!("{} min ago", secs / 60),
        3600..=86_399 => format!("{} h ago", secs / 3600),
        86_400..=172_799 => "yesterday".to_string(),
        _ => format!("{} days ago", secs / 86_400),
    }
}

/// A filesystem-safe lowercase slug for a title; "profile" when empty.
pub fn slugify(title: &str) -> String {
    let slug = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        "profile".to_string()
    } else {
        slug
    }
}

/// Pick a free profile id/dir under `root` from `title`, suffixing `-2`, `-3`…
/// on collision. Pure (takes the root) so it is unit-testable.
pub fn allocate_dir(root: &Path, title: &str) -> (String, PathBuf) {
    let base = slugify(title);
    let mut id = base.clone();
    let mut n = 2;
    while root.join(&id).exists() {
        id = format!("{base}-{n}");
        n += 1;
    }
    let dir = root.join(&id);
    (id, dir)
}

/// Allocate a new profile id/dir under the real profiles root.
pub fn new_profile_dir(title: &str) -> Result<(String, PathBuf)> {
    Ok(allocate_dir(&super::paths::profiles_dir()?, title))
}

/// Duplicate a profile: create a new "<title> (copy)" profile directory next to
/// the others, copying a relative cover image. Returns the new directory.
pub fn duplicate(src_dir: &Path, profile: &Profile) -> Result<PathBuf> {
    let title = format!("{} (copy)", profile.title);
    let (id, dir) = new_profile_dir(&title)?;
    let mut copy = profile.clone();
    copy.id = id;
    copy.title = title;
    copy.last_played = None;

    fs::create_dir_all(&dir).with_context(|| format!("creating profile dir {}", dir.display()))?;
    if let Some(cover) = &copy.cover {
        if cover.is_relative() {
            let from = src_dir.join(cover);
            if from.exists() {
                let _ = fs::copy(&from, dir.join(cover));
            }
        }
    }
    copy.save(&dir)?;
    Ok(dir)
}

/// File names of DOS executables (`.exe`/`.bat`/`.com`) directly in `dir`,
/// sorted case-insensitively. Used by the new-profile wizard.
pub fn scan_executables(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_lowercase);
        if matches!(ext.as_deref(), Some("exe" | "bat" | "com")) {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                out.push(name.to_string());
            }
        }
    }
    out.sort_by_key(|s| s.to_lowercase());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Profile {
        Profile {
            id: "dune2".into(),
            title: "Dune II".into(),
            genre: Some("Strategy".into()),
            year: Some(1992),
            developer: Some("Westwood".into()),
            publisher: None,
            www: None,
            notes: None,
            cover: None,
            favorite: true,
            run: RunSpec {
                mounts: vec![Mount {
                    drive: 'C',
                    kind: MountKind::Directory,
                    path: "/games/dune2".into(),
                    label: None,
                }],
                working_drive: 'C',
                command: "DUNE2.EXE".into(),
                args: vec![],
                exit_after: true,
            },
            dosbox: DosboxConfig {
                cycles: Some("max".into()),
                ..Default::default()
            },
            last_played: Some(1_700_000_000),
        }
    }

    #[test]
    fn toml_round_trip_preserves_profile() {
        let p = sample();
        let text = toml::to_string_pretty(&p).unwrap();
        let back: Profile = toml::from_str(&text).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn save_then_load_round_trips_on_disk() {
        let tmp = tempfile::tempdir().unwrap();
        let p = sample();
        p.save(tmp.path()).unwrap();
        let back = Profile::load(tmp.path()).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn scan_finds_profiles_and_skips_non_profile_dirs() {
        let root = tempfile::tempdir().unwrap();
        sample().save(&root.path().join("dune2")).unwrap();
        fs::create_dir_all(root.path().join("empty")).unwrap();

        let found = scan(root.path()).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].1.id, "dune2");
    }

    #[test]
    fn scan_missing_root_is_empty() {
        let root = tempfile::tempdir().unwrap();
        let missing = root.path().join("nope");
        assert!(scan(&missing).unwrap().is_empty());
    }

    #[test]
    fn slugify_normalizes_titles() {
        assert_eq!(slugify("Dune II"), "dune-ii");
        assert_eq!(slugify("  Commander Keen 4!  "), "commander-keen-4");
        assert_eq!(slugify("X-COM: UFO Defense"), "x-com-ufo-defense");
        assert_eq!(slugify(""), "profile");
        assert_eq!(slugify("***"), "profile");
    }

    #[test]
    fn allocate_dir_suffixes_on_collision() {
        let root = tempfile::tempdir().unwrap();
        let (id, dir) = allocate_dir(root.path(), "Dune II");
        assert_eq!(id, "dune-ii");
        fs::create_dir_all(&dir).unwrap();

        let (id2, _) = allocate_dir(root.path(), "Dune II");
        assert_eq!(id2, "dune-ii-2");
    }

    #[test]
    fn humanize_since_buckets() {
        assert_eq!(humanize_since(100, 100), "just now");
        assert_eq!(humanize_since(1000, 700), "5 min ago");
        assert_eq!(humanize_since(10_000, 3_000), "1 h ago");
        assert_eq!(humanize_since(200_000, 100_000), "yesterday"); // ~27h
        assert_eq!(humanize_since(300_000, 100_000), "2 days ago");
        assert_eq!(humanize_since(50, 100), "just now"); // clock skew -> safe
    }

    #[test]
    fn scan_executables_finds_dos_programs() {
        let dir = tempfile::tempdir().unwrap();
        for name in [
            "DUNE2.EXE",
            "install.bat",
            "GO.COM",
            "readme.txt",
            "data.dat",
        ] {
            fs::write(dir.path().join(name), b"x").unwrap();
        }
        fs::create_dir(dir.path().join("SAVE.EXE")).unwrap(); // a dir, not a file

        let found = scan_executables(dir.path());
        assert_eq!(found, vec!["DUNE2.EXE", "GO.COM", "install.bat"]);
    }
}
