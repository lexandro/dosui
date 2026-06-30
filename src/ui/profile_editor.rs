//! Modal profile editor shell: assembles the tabs (General / Mounts & Run /
//! DOSBox) into a Notebook and handles Save. The tab widgets live in
//! `editor_general` and `editor_mounts`; the DOSBox tabs come from `dosbox_form`.

use std::path::PathBuf;
use std::rc::Rc;

use gtk::gio;
use gtk::prelude::*;
use gtk::{
    AlertDialog, ApplicationWindow, Box as GtkBox, Button, Label, Notebook, Orientation, Window,
};

use crate::config::profile::Profile;
use crate::ui::dosbox_form::DosboxForm;
use crate::ui::editor_general::{self, General};
use crate::ui::editor_mounts::{self, Run};

/// Unset DOSBox values in a profile inherit from the global defaults.
const INHERIT: &str = "(inherit)";

/// All editor input widgets, grouped per tab and read back on Save.
struct Fields {
    general: General,
    run: Run,
    dos: DosboxForm,
}

/// Open the editor for an existing profile stored in `dir`. New profiles are
/// created via the wizard ([`crate::ui::wizard`]); the editor only edits.
pub fn open_for_edit(
    parent: &ApplicationWindow,
    dir: PathBuf,
    profile: Profile,
    on_saved: Rc<dyn Fn()>,
) {
    let window = Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(format!("Edit — {}", profile.title))
        .default_width(620)
        .default_height(560)
        .build();

    let notebook = Notebook::new();
    notebook.set_vexpand(true);

    let (general_page, general) = editor_general::build(&profile, &window);
    notebook.append_page(&general_page, Some(&Label::new(Some("General"))));
    let (run_page, run) = editor_mounts::build(&profile, &window);
    notebook.append_page(&run_page, Some(&Label::new(Some("Mounts & Run"))));
    let dos = DosboxForm::new(&profile.dosbox, INHERIT, false);
    notebook.append_page(&dos.cpu_page, Some(&Label::new(Some("CPU"))));
    notebook.append_page(&dos.memory_page, Some(&Label::new(Some("Memory"))));
    notebook.append_page(&dos.graphics_page, Some(&Label::new(Some("Graphics"))));
    notebook.append_page(&dos.sound_page, Some(&Label::new(Some("Sound"))));
    notebook.append_page(&dos.input_page, Some(&Label::new(Some("Input"))));
    let advanced_index =
        notebook.append_page(&dos.advanced_page, Some(&Label::new(Some("Advanced"))));

    let fields = Rc::new(Fields { general, run, dos });
    let original = Rc::new(profile);

    // Refresh the read-only preview (the effective, merged conf) on Advanced.
    {
        let fields = fields.clone();
        let original = original.clone();
        notebook.connect_switch_page(move |_, _, index| {
            if index == advanced_index {
                let p = collect(&fields, &original);
                let effective = crate::config::defaults::load().merge(&p.dosbox);
                fields.dos.set_preview(&effective.render(&p.run));
            }
        });
    }

    let actions = action_bar();
    let outer = GtkBox::builder().orientation(Orientation::Vertical).build();
    outer.append(&notebook);
    outer.append(&actions.container);
    window.set_child(Some(&outer));
    window.set_default_widget(Some(&actions.save)); // Enter confirms

    // Expose Save/Cancel as app actions while open (driveable from outside).
    if let Some(app) = parent.application() {
        register_editor_actions(&app, &window, &actions.save, &actions.cancel);
    }

    {
        let window = window.clone();
        actions.cancel.connect_clicked(move |_| window.close());
    }
    {
        let window = window.clone();
        actions.save.connect_clicked(move |_| {
            let updated = collect(&fields, &original);
            match updated.save(&dir) {
                Ok(()) => {
                    on_saved();
                    window.close();
                }
                Err(e) => {
                    log::error!("saving profile failed: {e:#}");
                    show_error(&window, "Could not save profile", &e);
                }
            }
        });
    }

    window.present();
}

/// Read the editor widgets into a profile, preserving fields no tab edits.
fn collect(fields: &Fields, original: &Profile) -> Profile {
    let mut p = original.clone();
    fields.general.apply(&mut p);
    fields.run.apply(&mut p);
    p.dosbox = fields.dos.collect();
    p
}

struct ActionBar {
    container: GtkBox,
    cancel: Button,
    save: Button,
}

fn action_bar() -> ActionBar {
    let cancel = Button::with_label("Cancel");
    let save = Button::builder()
        .label("Save")
        .css_classes(["suggested-action"])
        .build();
    let container = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .halign(gtk::Align::End)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .build();
    container.append(&cancel);
    container.append(&save);
    ActionBar {
        container,
        cancel,
        save,
    }
}

/// Temporary `editor-save` / `editor-cancel` app actions that click the buttons,
/// removed when the editor closes (editors are modal, one at a time).
fn register_editor_actions(
    app: &gtk::Application,
    window: &Window,
    save: &Button,
    cancel: &Button,
) {
    for (name, button) in [("editor-save", save), ("editor-cancel", cancel)] {
        let action = gio::SimpleAction::new(name, None);
        let button = button.clone();
        action.connect_activate(move |_, _| button.emit_clicked());
        app.add_action(&action);
    }
    let app = app.clone();
    window.connect_close_request(move |_| {
        app.remove_action("editor-save");
        app.remove_action("editor-cancel");
        gtk::glib::Propagation::Proceed
    });
}

/// Show an error in a modal alert tied to the editor window.
fn show_error(window: &Window, message: &str, error: &anyhow::Error) {
    AlertDialog::builder()
        .message(message.to_string())
        .detail(format!("{error:#}"))
        .build()
        .show(Some(window));
}
