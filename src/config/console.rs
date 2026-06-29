//! The built-in "DOS Console" profile: a seed entry that drops the user at a
//! DOSBox prompt with a ready-to-use C: drive.
//!
//! Unlike a game profile this has an empty `run.command`, which the autoexec
//! generator treats as "console": mount the drives, switch to C:, and stay at
//! the prompt (see [`super::dosbox_conf`]). It is a normal profile on disk, so
//! the user can delete it; [`ensure`] recreates it on demand.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::paths;
use super::profile::{Mount, MountKind, Profile, RunSpec, PROFILE_FILE};

/// Stable id / on-disk directory name of the built-in console profile.
pub const CONSOLE_ID: &str = "dos-console";

/// Subdirectory (inside the profile dir) mounted as the console's C: drive.
const DRIVE_C: &str = "drive_c";

/// True when a profile drops straight to a DOS prompt (no program to run).
/// The grid/detail panes use this to show a terminal icon instead of a cover.
pub fn is_console(profile: &Profile) -> bool {
    profile.run.command.trim().is_empty()
}

/// Build the canonical console profile, mounting `drive_c` as the C: drive.
pub fn console_profile(drive_c: &Path) -> Profile {
    Profile {
        id: CONSOLE_ID.to_string(),
        title: "DOS Console".to_string(),
        genre: None,
        year: None,
        developer: None,
        publisher: None,
        www: None,
        notes: Some(
            "A bare DOSBox prompt with a ready C: drive. Drop files into the \
             profile's drive_c folder to reach them as C:\\."
                .to_string(),
        ),
        cover: None,
        favorite: false,
        last_played: None,
        run: RunSpec {
            mounts: vec![Mount {
                drive: 'C',
                kind: MountKind::Directory,
                path: drive_c.to_path_buf(),
                label: None,
            }],
            working_drive: 'C',
            command: String::new(), // empty -> console: stay at the prompt
            args: Vec::new(),
            exit_after: false,
        },
        dosbox: Default::default(),
    }
}

/// Ensure the console profile exists under the profiles dir, creating it (and
/// its C: drive folder) if missing. Idempotent — re-running just re-creates a
/// deleted C: folder. Returns the profile directory and whether it was freshly
/// written.
pub fn ensure() -> Result<(PathBuf, bool)> {
    let dir = paths::profiles_dir()?.join(CONSOLE_ID);
    let drive_c = dir.join(DRIVE_C);
    std::fs::create_dir_all(&drive_c)
        .with_context(|| format!("creating console C: drive {}", drive_c.display()))?;
    if dir.join(PROFILE_FILE).exists() {
        return Ok((dir, false));
    }
    console_profile(&drive_c).save(&dir)?;
    Ok((dir, true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn console_profile_is_a_bare_c_prompt() {
        let p = console_profile(Path::new("/data/dos-console/drive_c"));
        assert_eq!(p.id, CONSOLE_ID);
        assert!(is_console(&p), "empty command means console");
        assert!(!p.run.exit_after, "console stays open at the prompt");
        assert_eq!(p.run.working_drive, 'C');
        assert_eq!(p.run.mounts.len(), 1);
        let c = &p.run.mounts[0];
        assert_eq!(c.drive, 'C');
        assert_eq!(c.kind, MountKind::Directory);
        assert_eq!(c.path, Path::new("/data/dos-console/drive_c"));
    }

    #[test]
    fn a_normal_profile_is_not_a_console() {
        let mut p = console_profile(Path::new("/x"));
        p.run.command = "DUNE2.EXE".to_string();
        assert!(!is_console(&p));
    }
}
