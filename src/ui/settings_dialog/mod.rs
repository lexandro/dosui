//! Settings dialog: edit the global DOSBox defaults and the DOSBox binary path.
//!
//! The DOSBox tabs reuse [`DosboxForm`] (same widget set as the profile editor,
//! but editing the global defaults that profiles inherit). The Application tab
//! lives in [`app_tab`]. Save writes `defaults.toml` and `settings.toml`.

mod app_tab;

use std::rc::Rc;

use gtk::gio;
use gtk::prelude::*;
use gtk::{ApplicationWindow, Box as GtkBox, Button, Entry, Label, Notebook, Orientation, Window};

use crate::config::defaults;
use crate::config::settings::AppSettings;
use crate::ui::dialogs;
use crate::ui::dosbox_form::DosboxForm;

/// Open the settings dialog.
///
/// Takes no "saved" callback: both files it writes (`defaults.toml`,
/// `settings.toml`) are re-read on demand — at launch and when the editor builds
/// its preview — so nothing in the library view goes stale.
pub fn open(parent: &ApplicationWindow) {
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

    let (app_page, dosbox_path) = app_tab::build(&settings, &window);
    notebook.append_page(&app_page, Some(&Label::new(Some("Application"))));

    let form = DosboxForm::new(&defaults, "(default)", true);
    notebook.append_page(&form.cpu_page, Some(&Label::new(Some("CPU"))));
    notebook.append_page(&form.memory_page, Some(&Label::new(Some("Memory"))));
    notebook.append_page(&form.graphics_page, Some(&Label::new(Some("Graphics"))));
    notebook.append_page(&form.sound_page, Some(&Label::new(Some("Sound"))));
    notebook.append_page(&form.input_page, Some(&Label::new(Some("Input"))));
    notebook.append_page(&form.dosenv_page, Some(&Label::new(Some("DOS"))));
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
                dialogs::error(&window, "Could not save settings", &e);
                return;
            }
            window.close();
        });
    }

    window.present();
}

/// Persist both the app settings and the global DOSBox defaults.
fn save_all(dosbox_path: &Entry, form: &DosboxForm) -> anyhow::Result<()> {
    let path = dosbox_path.text().trim().to_string();
    // Load-then-update so we preserve fields the dialog doesn't edit
    // (e.g. desktop_prompted).
    let mut settings = AppSettings::load();
    settings.dosbox_path = if path.is_empty() {
        None
    } else {
        Some(path.into())
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
