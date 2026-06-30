//! Import commands: a `dosbox.conf` file, a zipped game, and drag-and-drop.

use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gio, Application, ApplicationWindow, Box as GtkBox, DropTarget, FileDialog};

use super::report;
use crate::config::profile::{self, Profile};
use crate::config::{archive, conf_import};

/// Register the `import` and `import-zip` actions on `app`.
pub(super) fn register(app: &Application, window: &ApplicationWindow, reload: &Rc<dyn Fn()>) {
    let import = gio::SimpleAction::new("import", None);
    {
        let window = window.downgrade();
        let reload = reload.clone();
        import.connect_activate(move |_, _| {
            let Some(window) = window.upgrade() else {
                return;
            };
            let dialog = FileDialog::builder().title("Import dosbox.conf").build();
            let reload = reload.clone();
            let parent = window.clone();
            dialog.open(Some(&window), gio::Cancellable::NONE, move |res| {
                let Ok(file) = res else { return };
                let Some(path) = file.path() else { return };
                let title = path
                    .parent()
                    .and_then(|p| p.file_name())
                    .or_else(|| path.file_stem())
                    .and_then(|s| s.to_str())
                    .unwrap_or("Imported game")
                    .to_string();
                match std::fs::read_to_string(&path) {
                    Ok(text) => {
                        if let Err(e) = save_imported(&text, &title) {
                            report(&parent, "Could not import dosbox.conf", &e);
                        }
                        reload();
                    }
                    Err(e) => log::error!("reading {} failed: {e:#}", path.display()),
                }
            });
        });
    }
    app.add_action(&import);

    let import_zip = gio::SimpleAction::new("import-zip", None);
    {
        let window = window.downgrade();
        let reload = reload.clone();
        import_zip.connect_activate(move |_, _| {
            let Some(window) = window.upgrade() else {
                return;
            };
            let dialog = FileDialog::builder().title("Import zipped game").build();
            let reload = reload.clone();
            let parent = window.clone();
            dialog.open(Some(&window), gio::Cancellable::NONE, move |res| {
                let Ok(file) = res else { return };
                let Some(path) = file.path() else { return };
                if let Err(e) = archive::import_archive(&path) {
                    report(&parent, "Could not import archive", &e);
                }
                reload();
            });
        });
    }
    app.add_action(&import_zip);
}

/// Accept dropped `.zip` archives anywhere on `widget`: import each as a profile.
pub(crate) fn install_drop_target(widget: &GtkBox, reload: &Rc<dyn Fn()>) {
    let drop = DropTarget::new(
        gtk::gdk::FileList::static_type(),
        gtk::gdk::DragAction::COPY,
    );
    let reload = reload.clone();
    drop.connect_drop(move |_, value, _, _| {
        let Ok(list) = value.get::<gtk::gdk::FileList>() else {
            return false;
        };
        let mut imported = false;
        for file in list.files() {
            let Some(path) = file.path() else { continue };
            let is_zip = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("zip"));
            if is_zip {
                if let Err(e) = archive::import_archive(&path) {
                    log::error!("dropped zip import failed: {e:#}");
                }
                imported = true;
            }
        }
        if imported {
            reload();
        }
        imported
    });
    widget.add_controller(drop);
}

/// Import a dosbox.conf into a fresh profile directory.
fn save_imported(text: &str, title: &str) -> anyhow::Result<()> {
    let mut profile: Profile = conf_import::import_profile(text, title);
    let (id, dir) = profile::new_profile_dir(&profile.title)?;
    profile.id = id;
    profile.save(&dir)
}
