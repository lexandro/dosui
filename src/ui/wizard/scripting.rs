//! Temporary app actions mirroring the wizard controls, so the flow can be
//! driven without on-screen input (used by integration tests / `gapplication`).

use gtk::gio;
use gtk::prelude::*;
use gtk::{Button, Entry, Window};

/// Register `wizard-set-folder` (string path) plus `wizard-next` / `wizard-back`
/// / `wizard-cancel` (which click the buttons), all removed when `window` closes.
pub(super) fn register(
    app: &gtk::Application,
    window: &Window,
    folder: &Entry,
    back: &Button,
    next: &Button,
    cancel: &Button,
) {
    let set = gio::SimpleAction::new("wizard-set-folder", Some(gtk::glib::VariantTy::STRING));
    {
        let folder = folder.clone();
        set.connect_activate(move |_, param| {
            if let Some(path) = param.and_then(|v| v.str()) {
                folder.set_text(path);
            }
        });
    }
    app.add_action(&set);

    for (name, button) in [
        ("wizard-next", next),
        ("wizard-back", back),
        ("wizard-cancel", cancel),
    ] {
        let action = gio::SimpleAction::new(name, None);
        let button = button.clone();
        action.connect_activate(move |_, _| button.emit_clicked());
        app.add_action(&action);
    }

    let app = app.clone();
    window.connect_close_request(move |_| {
        for name in [
            "wizard-set-folder",
            "wizard-next",
            "wizard-back",
            "wizard-cancel",
        ] {
            app.remove_action(name);
        }
        gtk::glib::Propagation::Proceed
    });
}
