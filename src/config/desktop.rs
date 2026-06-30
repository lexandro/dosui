//! Desktop integration: install a `.desktop` launcher (+ icon) into the user's
//! applications menu and onto their desktop, so a portable AppImage shows up in
//! the system like an installed app.
//!
//! This module is GTK-free and only writes/removes files; the orchestration and
//! the file-manager "trusted" flag (which needs GIO) live in
//! [`crate::integration`], and the user prompt lives in the UI layer.
//! Everything takes explicit directories so it is unit-testable against temp
//! dirs. The embedded `.desktop` has its `Exec=` line replaced with the path of
//! the running binary at install time.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

const APP_ID: &str = "io.github.dosui";

const DESKTOP_TEMPLATE: &str = include_str!("../../data/io.github.dosui.desktop");
const ICON_PNG: &[u8] = include_bytes!("../../data/icons/hicolor/256x256/apps/io.github.dosui.png");

/// The command the `.desktop` `Exec=` should launch: prefer `$APPIMAGE` (the
/// portable file the user double-clicks) over the unpacked inner binary, which
/// only works while the AppImage is mounted. `None` if neither is resolvable.
pub fn exec_path() -> Option<PathBuf> {
    if let Some(appimage) = std::env::var_os("APPIMAGE") {
        return Some(PathBuf::from(appimage));
    }
    std::env::current_exe().ok()
}

fn menu_entry_path(data_home: &Path) -> PathBuf {
    data_home
        .join("applications")
        .join(format!("{APP_ID}.desktop"))
}

fn desktop_launcher_path(desktop_dir: &Path) -> PathBuf {
    desktop_dir.join(format!("{APP_ID}.desktop"))
}

/// Is the applications-menu entry already installed?
pub fn menu_entry_present(data_home: &Path) -> bool {
    menu_entry_path(data_home).exists()
}

/// Is the desktop-surface launcher already installed?
pub fn desktop_launcher_present(desktop_dir: &Path) -> bool {
    desktop_launcher_path(desktop_dir).exists()
}

/// Install the applications-menu entry and its icon under `data_home`
/// (`~/.local/share`). Overwrites an existing entry to refresh `Exec=`.
pub fn install_menu(data_home: &Path, exec: &Path) -> Result<PathBuf> {
    let icon_dir = data_home.join("icons/hicolor/256x256/apps");
    fs::create_dir_all(&icon_dir).with_context(|| format!("creating {}", icon_dir.display()))?;
    fs::write(icon_dir.join(format!("{APP_ID}.png")), ICON_PNG).context("writing icon")?;

    let path = menu_entry_path(data_home);
    fs::create_dir_all(path.parent().unwrap())
        .with_context(|| format!("creating {}", path.display()))?;
    fs::write(&path, desktop_contents(exec)).context("writing menu entry")?;
    Ok(path)
}

/// Install a launcher onto the user's desktop, marked executable so the desktop
/// treats it as an app. (The file-manager "trusted" flag is set by the caller,
/// which needs GIO.) Returns the written path.
pub fn install_desktop_launcher(desktop_dir: &Path, exec: &Path) -> Result<PathBuf> {
    fs::create_dir_all(desktop_dir)
        .with_context(|| format!("creating {}", desktop_dir.display()))?;
    let path = desktop_launcher_path(desktop_dir);
    fs::write(&path, desktop_contents(exec)).context("writing desktop launcher")?;
    make_executable(&path)?;
    Ok(path)
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perm = fs::metadata(path)?.permissions();
    perm.set_mode(0o755);
    fs::set_permissions(path, perm).with_context(|| format!("chmod +x {}", path.display()))
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<()> {
    Ok(())
}

/// Remove the applications-menu entry and its icon. Returns whether anything
/// was actually removed.
pub fn remove_menu(data_home: &Path) -> Result<bool> {
    let entry = remove_if_present(&menu_entry_path(data_home))?;
    let icon =
        remove_if_present(&data_home.join(format!("icons/hicolor/256x256/apps/{APP_ID}.png")))?;
    // Clean up the icon from older versions that shipped an SVG.
    let _ = remove_if_present(&data_home.join(format!("icons/hicolor/scalable/apps/{APP_ID}.svg")));
    Ok(entry || icon)
}

/// Remove the desktop-surface launcher. Returns whether it was present.
pub fn remove_desktop_launcher(desktop_dir: &Path) -> Result<bool> {
    remove_if_present(&desktop_launcher_path(desktop_dir))
}

/// Delete `path` if it exists; `Ok(false)` when it was already absent.
fn remove_if_present(path: &Path) -> Result<bool> {
    match fs::remove_file(path) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e).with_context(|| format!("removing {}", path.display())),
    }
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
    fn install_menu_writes_entry_and_icon() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        assert!(!menu_entry_present(home));

        install_menu(home, Path::new("/opt/dosui.AppImage")).unwrap();
        assert!(menu_entry_present(home));
        assert!(home
            .join("icons/hicolor/256x256/apps/io.github.dosui.png")
            .exists());
        assert!(fs::read_to_string(menu_entry_path(home))
            .unwrap()
            .contains("Exec=/opt/dosui.AppImage"));
    }

    #[test]
    fn remove_menu_deletes_entry_and_icon() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        install_menu(home, Path::new("/opt/dosui.AppImage")).unwrap();
        assert!(menu_entry_present(home));

        assert!(remove_menu(home).unwrap(), "removed something");
        assert!(!menu_entry_present(home));
        assert!(!home
            .join("icons/hicolor/256x256/apps/io.github.dosui.png")
            .exists());
        assert!(!remove_menu(home).unwrap(), "second remove is a no-op");
    }

    #[test]
    fn remove_desktop_launcher_deletes_it() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        install_desktop_launcher(dir, Path::new("/opt/dosui.AppImage")).unwrap();
        assert!(desktop_launcher_present(dir));

        assert!(remove_desktop_launcher(dir).unwrap());
        assert!(!desktop_launcher_present(dir));
        assert!(!remove_desktop_launcher(dir).unwrap());
    }

    #[test]
    #[cfg(unix)]
    fn desktop_launcher_is_executable() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        assert!(!desktop_launcher_present(dir));

        let path = install_desktop_launcher(dir, Path::new("/opt/dosui.AppImage")).unwrap();
        assert!(desktop_launcher_present(dir));
        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o111, 0o111, "launcher must be executable");
    }
}
