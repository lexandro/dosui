//! First-run desktop integration.
//!
//! AppImages aren't registered with the desktop on their own. When dosui runs
//! as a portable AppImage and has no menu entry yet, we install a `.desktop`
//! launcher plus its icon into the user's XDG data dirs so it shows up in the
//! application menu. Idempotent (only when missing) and best-effort — the
//! caller logs any error; it never blocks startup.
//!
//! GTK-free and unit-testable: everything takes an explicit `data_home`, so the
//! tests run against a temp directory.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

const APP_ID: &str = "io.github.dosui";

/// The canonical desktop entry and icon, embedded so we can always write them
/// out regardless of the runtime layout. The `Exec=` line is replaced at
/// install time with the path of the running binary.
const DESKTOP_TEMPLATE: &str = include_str!("../../data/io.github.dosui.desktop");
const ICON_SVG: &str = include_str!("../../data/io.github.dosui.svg");

/// The command the `.desktop` `Exec=` should launch: prefer `$APPIMAGE` (the
/// portable file the user double-clicks) over the unpacked inner binary, which
/// only works while the AppImage is mounted. `None` if neither is resolvable.
pub fn exec_path() -> Option<PathBuf> {
    if let Some(appimage) = std::env::var_os("APPIMAGE") {
        return Some(PathBuf::from(appimage));
    }
    std::env::current_exe().ok()
}

/// Install the launcher + icon if no user menu entry exists yet. Returns
/// `Ok(true)` when it installed, `Ok(false)` when an entry was already present.
pub fn ensure_first_run(data_home: &Path, exec: &Path) -> Result<bool> {
    if desktop_file(data_home).exists() {
        return Ok(false);
    }
    install(data_home, exec)?;
    Ok(true)
}

fn desktop_file(data_home: &Path) -> PathBuf {
    data_home
        .join("applications")
        .join(format!("{APP_ID}.desktop"))
}

fn install(data_home: &Path, exec: &Path) -> Result<()> {
    let icon_dir = data_home.join("icons/hicolor/scalable/apps");
    fs::create_dir_all(&icon_dir).with_context(|| format!("creating {}", icon_dir.display()))?;
    fs::write(icon_dir.join(format!("{APP_ID}.svg")), ICON_SVG).context("writing icon")?;

    let desktop = desktop_file(data_home);
    fs::create_dir_all(desktop.parent().unwrap())
        .with_context(|| format!("creating {}", desktop.display()))?;
    fs::write(&desktop, desktop_contents(exec)).context("writing desktop entry")?;
    Ok(())
}

/// The embedded `.desktop` with its `Exec=` line pointed at `exec`. Paths with
/// spaces are quoted, as the desktop-entry spec requires.
fn desktop_contents(exec: &Path) -> String {
    let path = exec.display().to_string();
    let value = if path.contains(' ') {
        format!("\"{path}\"")
    } else {
        path
    };
    let mut out: String = DESKTOP_TEMPLATE
        .lines()
        .map(|line| {
            if line.starts_with("Exec=") {
                format!("Exec={value}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_contents_swaps_exec_and_keeps_the_rest() {
        let out = desktop_contents(Path::new("/opt/dosui.AppImage"));
        assert!(out.contains("\nExec=/opt/dosui.AppImage\n"));
        assert!(!out.contains("Exec=dosui\n")); // the template's placeholder is gone
        assert!(out.contains("Icon=io.github.dosui"));
        assert!(out.contains("Categories=Game;Emulator;"));
    }

    #[test]
    fn exec_with_spaces_is_quoted() {
        let out = desktop_contents(Path::new("/home/a b/dosui.AppImage"));
        assert!(out.contains("Exec=\"/home/a b/dosui.AppImage\""));
    }

    #[test]
    fn ensure_first_run_installs_then_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let exec = Path::new("/opt/dosui.AppImage");

        assert!(ensure_first_run(home, exec).unwrap(), "first run installs");
        assert!(desktop_file(home).exists());
        assert!(home
            .join("icons/hicolor/scalable/apps/io.github.dosui.svg")
            .exists());
        assert!(fs::read_to_string(desktop_file(home))
            .unwrap()
            .contains("Exec=/opt/dosui.AppImage"));

        assert!(
            !ensure_first_run(home, exec).unwrap(),
            "second run is a no-op"
        );
    }
}
