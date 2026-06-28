//! Settings dialog: edit the global DOSBox defaults and the DOSBox binary path.
//!
//! The DOSBox tabs reuse [`DosboxForm`] (same widget set as the profile editor,
//! but editing the global defaults that profiles inherit). Save writes
//! `defaults.toml` and `settings.toml`.

use std::rc::Rc;

use gtk::gio;
use gtk::prelude::*;
use gtk::{
    AlertDialog, ApplicationWindow, Box as GtkBox, Button, Entry, FileDialog, Label, Notebook,
    Orientation, Window,
};

use crate::config::defaults;
use crate::config::settings::AppSettings;
use crate::ui::dosbox_form::DosboxForm;
use crate::ui::widgets;

/// Open the settings dialog. `on_saved` runs after a successful save.
pub fn open(parent: &ApplicationWindow, on_saved: Rc<dyn Fn()>) {
    let settings = AppSettings::load();
    let defaults = defaults::load();

    let window = Window::builder()
        .transient_for(parent)
        .modal(true)
        .title("Settings")
        .default_width(620)
        .default_height(560)
        .build();

    let notebook = Notebook::new();
    notebook.set_vexpand(true);

    let (app_page, dosbox_path) = build_app_tab(&settings, &window);
    notebook.append_page(&app_page, Some(&Label::new(Some("Application"))));

    let form = DosboxForm::new(&defaults, "(default)");
    notebook.append_page(&form.cpu_page, Some(&Label::new(Some("CPU"))));
    notebook.append_page(&form.graphics_page, Some(&Label::new(Some("Graphics"))));
    notebook.append_page(&form.sound_page, Some(&Label::new(Some("Sound"))));
    let advanced_index =
        notebook.append_page(&form.advanced_page, Some(&Label::new(Some("Advanced"))));

    let form = Rc::new(form);

    // Preview the generated defaults (no autoexec) when Advanced is shown.
    {
        let form = form.clone();
        notebook.connect_switch_page(move |_, _, index| {
            if index == advanced_index {
                let run = crate::config::profile::RunSpec {
                    mounts: Vec::new(),
                    working_drive: 'C',
                    command: String::new(),
                    args: Vec::new(),
                    exit_after: false,
                };
                form.set_preview(&form.collect().render(&run));
            }
        });
    }

    let cancel = Button::with_label("Cancel");
    let save = Button::builder()
        .label("Save")
        .css_classes(["suggested-action"])
        .build();
    let actions = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .halign(gtk::Align::End)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .build();
    actions.append(&cancel);
    actions.append(&save);

    let outer = GtkBox::builder().orientation(Orientation::Vertical).build();
    outer.append(&notebook);
    outer.append(&actions);
    window.set_child(Some(&outer));
    window.set_default_widget(Some(&save));

    if let Some(app) = parent.application() {
        register_actions(&app, &window, &save, &cancel);
    }

    {
        let window = window.clone();
        cancel.connect_clicked(move |_| window.close());
    }
    {
        let window = window.clone();
        save.connect_clicked(move |_| {
            if let Err(e) = save_all(&dosbox_path, &form) {
                log::error!("saving settings failed: {e:#}");
                AlertDialog::builder()
                    .message("Could not save settings")
                    .detail(format!("{e:#}"))
                    .build()
                    .show(Some(&window));
                return;
            }
            on_saved();
            window.close();
        });
    }

    window.present();
}

/// Application tab: the DOSBox binary path with a Browse and an Auto-detect button.
fn build_app_tab(settings: &AppSettings, window: &Window) -> (GtkBox, Entry) {
    let page = widgets::page();
    page.append(
        &Label::builder()
            .label("Leave the path empty to auto-detect dosbox-staging / dosbox on PATH.")
            .halign(gtk::Align::Start)
            .css_classes(["dim-label"])
            .build(),
    );
    let current = settings
        .dosbox_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let (row, dosbox_path, browse) = widgets::file_row("DOSBox binary", &current);
    page.append(&row);

    {
        let window = window.downgrade();
        let entry = dosbox_path.clone();
        browse.connect_clicked(move |_| {
            let dialog = FileDialog::builder().title("Select DOSBox binary").build();
            let entry = entry.clone();
            dialog.open(
                window.upgrade().as_ref(),
                gio::Cancellable::NONE,
                move |res| {
                    if let Ok(file) = res {
                        if let Some(path) = file.path() {
                            entry.set_text(&path.display().to_string());
                        }
                    }
                },
            );
        });
    }

    (page, dosbox_path)
}

/// Persist both the app settings and the global DOSBox defaults.
fn save_all(dosbox_path: &Entry, form: &DosboxForm) -> anyhow::Result<()> {
    let path = dosbox_path.text().trim().to_string();
    let settings = AppSettings {
        dosbox_path: if path.is_empty() {
            None
        } else {
            Some(path.into())
        },
    };
    settings.save()?;
    defaults::save(&form.collect())
}

/// Temporary `settings-save` / `settings-cancel` app actions (driveable from
/// outside), removed when the dialog closes.
fn register_actions(app: &gtk::Application, window: &Window, save: &Button, cancel: &Button) {
    for (name, button) in [("settings-save", save), ("settings-cancel", cancel)] {
        let action = gio::SimpleAction::new(name, None);
        let button = button.clone();
        action.connect_activate(move |_, _| button.emit_clicked());
        app.add_action(&action);
    }
    let app = app.clone();
    window.connect_close_request(move |_| {
        app.remove_action("settings-save");
        app.remove_action("settings-cancel");
        gtk::glib::Propagation::Proceed
    });
}
