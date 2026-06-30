//! Miscellaneous commands: settings, bulk-edit, about, quit.

use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gio, AlertDialog, Application, ApplicationWindow};

use crate::app::APP_NAME;
use crate::ui::library::Profiles;

/// Register the misc actions on `app`.
pub(super) fn register(
    app: &Application,
    window: &ApplicationWindow,
    profiles: &Profiles,
    reload: &Rc<dyn Fn()>,
) {
    let bulk_edit = gio::SimpleAction::new("bulk-edit", None);
    {
        let profiles = profiles.clone();
        let window = window.downgrade();
        let reload = reload.clone();
        bulk_edit.connect_activate(move |_, _| {
            if let Some(window) = window.upgrade() {
                crate::ui::bulk_edit::open(&window, profiles.borrow().clone(), reload.clone());
            }
        });
    }
    app.add_action(&bulk_edit);

    let settings = gio::SimpleAction::new("settings", None);
    {
        let window = window.downgrade();
        settings.connect_activate(move |_, _| {
            if let Some(window) = window.upgrade() {
                crate::ui::settings_dialog::open(&window, Rc::new(|| {}));
            }
        });
    }
    app.add_action(&settings);

    let about = gio::SimpleAction::new("about", None);
    {
        let window = window.downgrade();
        about.connect_activate(move |_, _| {
            if let Some(window) = window.upgrade() {
                AlertDialog::builder()
                    .modal(true)
                    .message(APP_NAME)
                    .detail("Lightweight native Linux frontend for DOSBox.\nRust + GTK4.")
                    .build()
                    .show(Some(&window));
            }
        });
    }
    app.add_action(&about);

    let quit = gio::SimpleAction::new("quit", None);
    {
        let app = app.clone();
        quit.connect_activate(move |_, _| app.quit());
    }
    app.add_action(&quit);
}
