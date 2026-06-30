//! Profile lifecycle commands: play, edit, new, add-console, duplicate, delete,
//! open-folder, favorite.

use std::path::Path;
use std::rc::Rc;

use gtk::glib::BoxedAnyObject;
use gtk::prelude::*;
use gtk::{gio, AlertDialog, Application, ApplicationWindow, FileLauncher, SingleSelection};

use super::report;
use crate::config::console;
use crate::config::profile::{self, Profile};
use crate::launcher;
use crate::ui::library::{selected_entry, Entry};

/// Register the lifecycle actions on `app`.
pub(super) fn register(
    app: &Application,
    window: &ApplicationWindow,
    selection: &SingleSelection,
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

    app.add_action(&delete_action(window, selection, reload));

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
}

/// The `delete` action: confirm, then remove the profile directory.
fn delete_action(
    window: &ApplicationWindow,
    selection: &SingleSelection,
    reload: &Rc<dyn Fn()>,
) -> gio::SimpleAction {
    let delete = gio::SimpleAction::new("delete", None);
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
    delete
}

/// Select the entry whose profile id matches `id`, if present (and not filtered
/// out). Used after adding the console so the new tile is highlighted.
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

/// Launch a profile, reporting failures in a dialog.
fn launch_profile(dir: &Path, profile: &Profile, window: Option<ApplicationWindow>) {
    if let Err(e) = launcher::launch(dir, profile) {
        log::error!("launch failed: {e:#}");
        if let Some(window) = window {
            report(&window, &format!("Could not launch {}", profile.title), &e);
        }
    }
}
