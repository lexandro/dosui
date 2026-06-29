//! All application command actions in one place.
//!
//! Over the 150-line soft cap by design: this is the single source of truth for
//! every command (menu / toolbar / keyboard / context menu / external
//! `gapplication`), and keeping the wiring together is clearer than scattering
//! related commands across files.

use std::path::Path;
use std::rc::Rc;

use gtk::glib::BoxedAnyObject;
use gtk::prelude::*;
use gtk::{
    gio, AlertDialog, Application, ApplicationWindow, Box as GtkBox, DropTarget, FileDialog,
    FileLauncher, SingleSelection,
};

use crate::app::APP_NAME;
use crate::config::profile::{self, Profile};
use crate::config::{archive, conf_import, console};
use crate::launcher;
use crate::ui::library::{selected_entry, Entry, Profiles};

/// Register every `app.*` action plus accelerators.
pub(crate) fn install_actions(
    app: &Application,
    window: &ApplicationWindow,
    selection: &SingleSelection,
    profiles: &Profiles,
    reload: &Rc<dyn Fn()>,
) {
    let play = gio::SimpleAction::new("play", None);
    {
        let selection = selection.clone();
        let window = window.downgrade();
        play.connect_activate(move |_, _| {
            if let Some((dir, profile)) = selected_entry(&selection) {
                launch_profile(&dir, &profile, window.upgrade());
            }
        });
    }
    app.add_action(&play);

    let edit = gio::SimpleAction::new("edit", None);
    {
        let selection = selection.clone();
        let window = window.downgrade();
        let reload = reload.clone();
        edit.connect_activate(move |_, _| {
            let Some((dir, prof)) = selected_entry(&selection) else {
                return;
            };
            if let Some(window) = window.upgrade() {
                crate::ui::profile_editor::open_for_edit(&window, dir, prof, reload.clone());
            }
        });
    }
    app.add_action(&edit);

    let new = gio::SimpleAction::new("new", None);
    {
        let window = window.downgrade();
        let reload = reload.clone();
        new.connect_activate(move |_, _| {
            if let Some(window) = window.upgrade() {
                crate::ui::wizard::open(&window, reload.clone());
            }
        });
    }
    app.add_action(&new);

    // Recreate the built-in DOS console profile (if a user deleted it) and
    // select it. Idempotent: an existing console is simply re-selected.
    let add_console = gio::SimpleAction::new("add-console", None);
    {
        let selection = selection.clone();
        let window = window.downgrade();
        let reload = reload.clone();
        add_console.connect_activate(move |_, _| match console::ensure() {
            Ok(_) => {
                reload();
                select_by_id(&selection, console::CONSOLE_ID);
            }
            Err(e) => {
                log::error!("adding DOS console failed: {e:#}");
                if let Some(window) = window.upgrade() {
                    report(&window, "Could not add DOS console", &e);
                }
            }
        });
    }
    app.add_action(&add_console);

    install_import_actions(app, window, reload);

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

    let delete = gio::SimpleAction::new("delete", None);
    {
        let selection = selection.clone();
        let window = window.downgrade();
        let reload = reload.clone();
        delete.connect_activate(move |_, _| {
            let Some((dir, prof)) = selected_entry(&selection) else {
                return;
            };
            let Some(window) = window.upgrade() else {
                return;
            };
            let reload = reload.clone();
            let dialog = AlertDialog::builder()
                .modal(true)
                .message(format!("Delete “{}”?", prof.title))
                .detail("This removes the profile. The game files are not touched.")
                .buttons(["Cancel", "Delete"])
                .cancel_button(0)
                .default_button(0)
                .build();
            dialog.choose(Some(&window), gio::Cancellable::NONE, move |res| {
                if res == Ok(1) {
                    if let Err(e) = std::fs::remove_dir_all(&dir) {
                        log::error!("deleting profile failed: {e:#}");
                    }
                    reload();
                }
            });
        });
    }
    app.add_action(&delete);

    let duplicate = gio::SimpleAction::new("duplicate", None);
    {
        let selection = selection.clone();
        let reload = reload.clone();
        duplicate.connect_activate(move |_, _| {
            let Some((dir, prof)) = selected_entry(&selection) else {
                return;
            };
            if let Err(e) = profile::duplicate(&dir, &prof) {
                log::error!("duplicating profile failed: {e:#}");
            }
            reload();
        });
    }
    app.add_action(&duplicate);

    let open_folder = gio::SimpleAction::new("open-folder", None);
    {
        let selection = selection.clone();
        let window = window.downgrade();
        open_folder.connect_activate(move |_, _| {
            let Some((dir, _)) = selected_entry(&selection) else {
                return;
            };
            let launcher = FileLauncher::new(Some(&gio::File::for_path(&dir)));
            launcher.launch(window.upgrade().as_ref(), gio::Cancellable::NONE, |res| {
                if let Err(e) = res {
                    log::warn!("opening folder failed: {e}");
                }
            });
        });
    }
    app.add_action(&open_folder);

    let favorite = gio::SimpleAction::new("favorite", None);
    {
        let selection = selection.clone();
        let reload = reload.clone();
        favorite.connect_activate(move |_, _| {
            let Some((dir, mut prof)) = selected_entry(&selection) else {
                return;
            };
            prof.favorite = !prof.favorite;
            if let Err(e) = prof.save(&dir) {
                log::error!("toggling favorite failed: {e:#}");
            }
            reload();
        });
    }
    app.add_action(&favorite);

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

    enable_with_selection(
        selection,
        &[play, edit, duplicate, delete, open_folder, favorite],
    );
    set_accels(app);
}

/// Select the entry whose profile id matches `id`, if it is present (and not
/// filtered out). Used after adding the console so the new tile is highlighted.
fn select_by_id(selection: &SingleSelection, id: &str) {
    let Some(model) = selection.model() else {
        return;
    };
    for i in 0..model.n_items() {
        let Some(obj) = model.item(i).and_downcast::<BoxedAnyObject>() else {
            continue;
        };
        if obj.borrow::<Entry>().1.id == id {
            selection.set_selected(i);
            return;
        }
    }
}

/// Disable selection-dependent commands when nothing is selected.
fn enable_with_selection(selection: &SingleSelection, actions: &[gio::SimpleAction]) {
    let actions: Vec<gio::SimpleAction> = actions.to_vec();
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

/// `import` (dosbox.conf) and `import-zip` actions.
fn install_import_actions(app: &Application, window: &ApplicationWindow, reload: &Rc<dyn Fn()>) {
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

/// Launch a profile, reporting failures in a dialog.
fn launch_profile(dir: &Path, profile: &Profile, window: Option<ApplicationWindow>) {
    if let Err(e) = launcher::launch(dir, profile) {
        log::error!("launch failed: {e:#}");
        if let Some(window) = window {
            report(&window, &format!("Could not launch {}", profile.title), &e);
        }
    }
}

/// Import a dosbox.conf into a fresh profile directory.
fn save_imported(text: &str, title: &str) -> anyhow::Result<()> {
    let mut profile = conf_import::import_profile(text, title);
    let (id, dir) = profile::new_profile_dir(&profile.title)?;
    profile.id = id;
    profile.save(&dir)
}

/// Show an error in a modal alert.
fn report(window: &ApplicationWindow, message: &str, error: &anyhow::Error) {
    AlertDialog::builder()
        .message(message.to_string())
        .detail(format!("{error:#}"))
        .build()
        .show(Some(window));
}
