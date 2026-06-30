//! Desktop-shortcut integration: install or remove the user's applications-menu
//! and desktop launchers. Orchestrates the GTK-free [`crate::config::desktop`]
//! file work and uses GIO only to flag the desktop launcher trusted.
//!
//! Callable headless (the `--install` / `--uninstall` CLI) and from the UI — it
//! needs no GTK widgets or running main loop, just GIO (part of GLib).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use gtk::gio;
use gtk::prelude::*;

use crate::config::{desktop, paths};

/// Are we running as a portable AppImage (where user-level shortcuts apply)?
pub fn is_appimage() -> bool {
    std::env::var_os("APPIMAGE").is_some()
}

/// Are the user-level shortcuts (menu or desktop) currently present?
pub fn is_installed() -> bool {
    let menu = paths::data_home()
        .map(|h| desktop::menu_entry_present(&h))
        .unwrap_or(false);
    let surface = desktop_dir()
        .map(|d| desktop::desktop_launcher_present(&d))
        .unwrap_or(false);
    menu || surface
}

/// Install the menu entry (+ icon) and, if the user has a desktop directory, a
/// desktop launcher marked executable and trusted.
pub fn install() -> Result<()> {
    let exec = desktop::exec_path().context("could not resolve the running executable")?;
    let data_home = paths::data_home()?;
    desktop::install_menu(&data_home, &exec)?;
    if let Some(dir) = desktop_dir() {
        let launcher = desktop::install_desktop_launcher(&dir, &exec)?;
        mark_trusted(&launcher);
    }
    Ok(())
}

/// Remove the menu entry, its icon, and the desktop launcher. Returns whether
/// anything was actually removed.
pub fn uninstall() -> Result<bool> {
    let data_home = paths::data_home()?;
    let mut removed = desktop::remove_menu(&data_home)?;
    if let Some(dir) = desktop_dir() {
        removed |= desktop::remove_desktop_launcher(&dir)?;
    }
    Ok(removed)
}

/// The user's desktop directory (`$XDG_DESKTOP_DIR` or `~/Desktop`), if any.
fn desktop_dir() -> Option<PathBuf> {
    directories::UserDirs::new().and_then(|d| d.desktop_dir().map(Path::to_path_buf))
}

/// Flag a desktop `.desktop` as trusted so the file manager (Nautilus / Nemo)
/// launches it without an "untrusted launcher" warning. Best-effort.
fn mark_trusted(path: &Path) {
    let file = gio::File::for_path(path);
    if let Err(e) = file.set_attribute_string(
        "metadata::trusted",
        "true",
        gio::FileQueryInfoFlags::NONE,
        gio::Cancellable::NONE,
    ) {
        log::debug!("could not mark {} trusted: {e}", path.display());
    }
}
