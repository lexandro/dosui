//! Loading, saving, and scanning profiles on disk.
//!
//! Over the 150-line soft cap by design: over half is the inline test suite
//! covering round-trips and directory scanning.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::{Profile, PROFILE_FILE};

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
    use crate::config::dosbox_conf::DosboxConfig;
    use crate::config::profile::{Mount, MountKind, RunSpec};

    pub(super) fn sample() -> Profile {
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
