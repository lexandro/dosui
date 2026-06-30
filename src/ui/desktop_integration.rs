//! Desktop integration UX: ask the user (once) whether to add dosui to their
//! applications menu and desktop, and perform it. The file writing lives in the
//! GTK-free [`crate::config::desktop`]; this layer owns the prompt, the
//! remembered choice, and the file-manager "trusted" flag (which needs GIO).
//!
//! Only relevant when running as a portable AppImage (`$APPIMAGE` set); for
//! installed or dev runs the menu entry already exists (or shouldn't be made).

use std::path::{Path, PathBuf};

use anyhow::Context;
use gtk::prelude::*;
use gtk::{gio, AlertDialog, ApplicationWindow};

use crate::config::settings::AppSettings;
use crate::config::{desktop, paths};

/// On startup: if running as an AppImage that isn't fully integrated yet and we
/// haven't asked before, ask whether to add menu + desktop shortcuts.
pub fn maybe_prompt(window: &ApplicationWindow) {
    if std::env::var_os("APPIMAGE").is_none() {
        return;
    }
    let Ok(data_home) = paths::data_home() else {
        return;
    };
    let surface_present = desktop_dir()
        .as_deref()
        .map(desktop::desktop_launcher_present)
        .unwrap_or(true);
    if desktop::menu_entry_present(&data_home) && surface_present {
        return; // already integrated
    }
    if AppSettings::load().desktop_prompted {
        return; // already asked — never nag
    }

    let dialog = AlertDialog::builder()
        .modal(true)
        .message("Add dosui to your menu?")
        .detail(
            "dosui can add a shortcut to your applications menu and your desktop, \
             so you can launch it without finding the AppImage file each time. \
             You can add or change this later from Settings.",
        )
        .buttons(["Not now", "Add shortcuts"])
        .cancel_button(0)
        .default_button(1)
        .build();

    let owned = window.clone();
    dialog.choose(Some(window), gio::Cancellable::NONE, move |res| {
        remember_prompted();
        if res == Ok(1) {
            if let Err(e) = do_integrate() {
                report(&owned, "Could not add shortcuts", &e);
            }
        }
    });
}

/// Force integration now (the Settings button): always (re)writes the shortcuts
/// and confirms. Handles "I said no earlier" and "I moved the AppImage".
pub fn integrate_now(window: &impl IsA<gtk::Window>) {
    match do_integrate() {
        Ok(()) => {
            remember_prompted();
            AlertDialog::builder()
                .modal(true)
                .message("Shortcuts added")
                .detail("dosui was added to your applications menu and your desktop.")
                .build()
                .show(Some(window));
        }
        Err(e) => report(window, "Could not add shortcuts", &e),
    }
}

/// Write the menu entry (+ icon) and, if the user has a desktop directory, a
/// desktop launcher marked executable and trusted.
fn do_integrate() -> anyhow::Result<()> {
    let exec = desktop::exec_path().context("could not resolve the running executable")?;
    let data_home = paths::data_home()?;
    desktop::install_menu(&data_home, &exec)?;
    if let Some(dir) = desktop_dir() {
        let launcher = desktop::install_desktop_launcher(&dir, &exec)?;
        mark_trusted(&launcher);
    }
    Ok(())
}

/// The user's desktop directory (`$XDG_DESKTOP_DIR` or `~/Desktop`), if any.
fn desktop_dir() -> Option<PathBuf> {
    directories::UserDirs::new().and_then(|d| d.desktop_dir().map(Path::to_path_buf))
}

/// Mark a desktop `.desktop` file as trusted so the file manager (Nautilus /
/// Nemo) launches it without an "untrusted launcher" warning. Best-effort.
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

/// Persist that we've offered integration, so we never ask again.
fn remember_prompted() {
    let mut settings = AppSettings::load();
    if !settings.desktop_prompted {
        settings.desktop_prompted = true;
        if let Err(e) = settings.save() {
            log::warn!("could not record desktop_prompted: {e:#}");
        }
    }
}

fn report(window: &impl IsA<gtk::Window>, message: &str, error: &anyhow::Error) {
    AlertDialog::builder()
        .message(message.to_string())
        .detail(format!("{error:#}"))
        .build()
        .show(Some(window));
}
