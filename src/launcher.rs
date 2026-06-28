//! Launching the DOSBox engine for a profile.
//!
//! Flow (plan §2.4): regenerate the per-profile `dosbox.conf`, resolve the
//! DOSBox binary, then spawn it non-blockingly via `gio::Subprocess` with
//! `-conf`. Mounts and the run command live in `[autoexec]`, so this is
//! identical across dosbox-staging / dosbox-x / vanilla.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use gtk::gio;

use crate::config::profile::Profile;
use crate::config::settings::AppSettings;

/// Generated config file name written into each profile directory.
const CONF_FILE: &str = "dosbox.conf";

/// Regenerate the profile's `dosbox.conf` and launch DOSBox against it.
///
/// Returns once the process has been *spawned*; the game runs asynchronously so
/// the UI never blocks. Exit status is logged when DOSBox quits.
pub fn launch(profile_dir: &Path, profile: &Profile, settings: &AppSettings) -> Result<()> {
    let conf_path = write_conf(profile_dir, profile)?;
    let binary = resolve_dosbox(settings)?;
    spawn(&binary, &conf_path, profile_dir)
        .with_context(|| format!("launching {}", binary.display()))
}

/// Render and write `<profile_dir>/dosbox.conf`, returning its path.
fn write_conf(profile_dir: &Path, profile: &Profile) -> Result<PathBuf> {
    fs::create_dir_all(profile_dir)
        .with_context(|| format!("creating profile dir {}", profile_dir.display()))?;
    let conf = profile.dosbox.render(&profile.run);
    let conf_path = profile_dir.join(CONF_FILE);
    fs::write(&conf_path, conf).with_context(|| format!("writing {}", conf_path.display()))?;
    Ok(conf_path)
}

/// Find the DOSBox binary: explicit setting → bundled (AppImage) → PATH.
pub fn resolve_dosbox(settings: &AppSettings) -> Result<PathBuf> {
    if let Some(path) = &settings.dosbox_path {
        if path.exists() {
            return Ok(path.clone());
        }
        log::warn!("configured dosbox_path {} not found", path.display());
    }

    if let Ok(appdir) = std::env::var("APPDIR") {
        let bundled = Path::new(&appdir).join("usr/bin/dosbox");
        if bundled.exists() {
            return Ok(bundled);
        }
    }

    for name in ["dosbox-staging", "dosbox"] {
        if let Ok(found) = which::which(name) {
            return Ok(found);
        }
    }

    bail!("DOSBox not found — set its path in Settings or install dosbox-staging")
}

/// Spawn DOSBox with the given conf, with the profile dir as working directory.
fn spawn(binary: &Path, conf: &Path, cwd: &Path) -> Result<()> {
    let launcher = gio::SubprocessLauncher::new(gio::SubprocessFlags::NONE);
    launcher.set_cwd(cwd);

    let argv: [&OsStr; 3] = [binary.as_os_str(), OsStr::new("-conf"), conf.as_os_str()];
    let process = launcher
        .spawn(&argv)
        .map_err(|e| anyhow::anyhow!("spawn failed: {e}"))?;

    let label = binary.display().to_string();
    process.wait_async(gio::Cancellable::NONE, move |res| match res {
        Ok(()) => log::info!("{label} exited"),
        Err(e) => log::warn!("{label} wait failed: {e}"),
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_prefers_configured_existing_path() {
        let tmp = tempfile::tempdir().unwrap();
        let fake = tmp.path().join("dosbox");
        fs::write(&fake, b"#!/bin/true").unwrap();

        let settings = AppSettings {
            dosbox_path: Some(fake.clone()),
        };
        assert_eq!(resolve_dosbox(&settings).unwrap(), fake);
    }

    #[test]
    fn write_conf_creates_file_with_autoexec() {
        use crate::config::profile::{MountKind, RunSpec};
        let tmp = tempfile::tempdir().unwrap();
        let profile = Profile {
            id: "t".into(),
            title: "T".into(),
            genre: None,
            year: None,
            developer: None,
            publisher: None,
            www: None,
            notes: None,
            cover: None,
            favorite: false,
            run: RunSpec {
                mounts: vec![crate::config::profile::Mount {
                    drive: 'C',
                    kind: MountKind::Directory,
                    path: "/games/t".into(),
                    label: None,
                }],
                working_drive: 'C',
                command: "GO.EXE".into(),
                args: vec![],
                exit_after: true,
            },
            dosbox: Default::default(),
        };

        let path = write_conf(tmp.path(), &profile).unwrap();
        let written = fs::read_to_string(&path).unwrap();
        assert!(written.contains("[autoexec]"));
        assert!(written.contains("mount C \"/games/t\""));
        assert!(written.contains("GO.EXE"));
    }
}
