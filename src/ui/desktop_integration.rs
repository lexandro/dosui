//! Desktop integration UX: the first-run prompt, the remembered choice, and the
//! confirmation dialogs for the Settings Add/Remove buttons. The actual file
//! work lives in [`crate::integration`]; this layer is just GTK glue.
//!
//! Only relevant when running as a portable AppImage; for installed or dev runs
//! the menu entry is managed by the package, not by us.

use gtk::prelude::*;
use gtk::{gio, AlertDialog, ApplicationWindow};

use crate::config::settings::AppSettings;
use crate::integration;
use crate::ui::dialogs;

/// On startup: if running as an AppImage that isn't integrated yet and we
/// haven't asked before, ask whether to add menu + desktop shortcuts.
pub fn maybe_prompt(window: &ApplicationWindow) {
    if !integration::is_appimage() || integration::is_installed() {
        return;
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
             You can add or remove this later from Settings.",
        )
        .buttons(["Not now", "Add shortcuts"])
        .cancel_button(0)
        .default_button(1)
        .build();

    let owned = window.clone();
    dialog.choose(Some(window), gio::Cancellable::NONE, move |res| {
        remember_prompted();
        if res == Ok(1) {
            if let Err(e) = integration::install() {
                dialogs::error(&owned, "Could not add shortcuts", &e);
            }
        }
    });
}

/// Settings button: (re)install the shortcuts now and confirm.
pub fn integrate_now(window: &impl IsA<gtk::Window>) {
    match integration::install() {
        Ok(()) => {
            remember_prompted();
            dialogs::note(
                window,
                "Shortcuts added",
                "dosui is now in your applications menu and on your desktop.",
            );
        }
        Err(e) => dialogs::error(window, "Could not add shortcuts", &e),
    }
}

/// Settings button: remove the shortcuts now and confirm.
pub fn uninstall_now(window: &impl IsA<gtk::Window>) {
    match integration::uninstall() {
        Ok(true) => dialogs::note(
            window,
            "Shortcuts removed",
            "dosui was removed from your menu and desktop.",
        ),
        Ok(false) => dialogs::note(
            window,
            "Nothing to remove",
            "No dosui shortcuts were installed.",
        ),
        Err(e) => dialogs::error(window, "Could not remove shortcuts", &e),
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
