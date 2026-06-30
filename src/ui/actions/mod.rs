//! All application command actions, grouped by concern.
//!
//! [`install_actions`] registers every `app.*` command and its accelerator; the
//! commands live in submodules (`lifecycle` / `import` / `misc`). This module
//! wires them together and holds the cross-cutting bits (selection-gating,
//! accelerators, the shared error dialog).

mod import;
mod lifecycle;
mod misc;

use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gio, AlertDialog, Application, ApplicationWindow, SingleSelection};

use crate::ui::library::Profiles;

pub(crate) use import::install_drop_target;

/// Commands that only make sense with a profile selected.
const SELECTION_ACTIONS: [&str; 6] = [
    "play",
    "edit",
    "duplicate",
    "delete",
    "open-folder",
    "favorite",
];

/// Register every `app.*` action plus accelerators.
pub(crate) fn install_actions(
    app: &Application,
    window: &ApplicationWindow,
    selection: &SingleSelection,
    profiles: &Profiles,
    reload: &Rc<dyn Fn()>,
) {
    lifecycle::register(app, window, selection, reload);
    import::register(app, window, reload);
    misc::register(app, window, profiles, reload);

    enable_with_selection(app, selection, &SELECTION_ACTIONS);
    set_accels(app);
}

/// Disable [`SELECTION_ACTIONS`] (looked up by name) when nothing is selected.
fn enable_with_selection(app: &Application, selection: &SingleSelection, names: &[&str]) {
    let actions: Vec<gio::SimpleAction> = names
        .iter()
        .filter_map(|n| app.lookup_action(n).and_downcast())
        .collect();
    let update = {
        let selection = selection.clone();
        move || {
            let enabled = selection.selected_item().is_some();
            for action in &actions {
                action.set_enabled(enabled);
            }
        }
    };
    update();
    selection.connect_selected_notify(move |_| update());
}

fn set_accels(app: &Application) {
    app.set_accels_for_action("app.play", &["<Ctrl>p"]);
    app.set_accels_for_action("app.edit", &["<Ctrl>e"]);
    app.set_accels_for_action("app.new", &["<Ctrl>n"]);
    app.set_accels_for_action("app.add-console", &["<Ctrl>t"]);
    app.set_accels_for_action("app.import", &["<Ctrl>i"]);
    app.set_accels_for_action("app.duplicate", &["<Ctrl>d"]);
    app.set_accels_for_action("app.delete", &["Delete"]);
    app.set_accels_for_action("app.settings", &["<Ctrl>comma"]);
    app.set_accels_for_action("app.quit", &["<Ctrl>q"]);
}

/// Show an error in a modal alert. Shared by the action submodules.
pub(super) fn report(window: &ApplicationWindow, message: &str, error: &anyhow::Error) {
    AlertDialog::builder()
        .message(message.to_string())
        .detail(format!("{error:#}"))
        .build()
        .show(Some(window));
}
