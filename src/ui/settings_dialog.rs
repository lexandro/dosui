//! Settings dialog: edit the global DOSBox defaults and the DOSBox binary path.
//!
//! The DOSBox tabs reuse [`DosboxForm`] (same widget set as the profile editor,
//! but editing the global defaults that profiles inherit). Save writes
//! `defaults.toml` and `settings.toml`.

use std::rc::Rc;

use gtk::gio;
use gtk::prelude::*;
use gtk::{
    AlertDialog, ApplicationWindow, Box as GtkBox, Button, Entry, FileDialog, Frame, Label,
    Notebook, Orientation, SizeGroup, SizeGroupMode, Widget, Window,
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

    let form = DosboxForm::new(&defaults, "(default)", true);
    notebook.append_page(&form.cpu_page, Some(&Label::new(Some("CPU"))));
    notebook.append_page(&form.memory_page, Some(&Label::new(Some("Memory"))));
    notebook.append_page(&form.graphics_page, Some(&Label::new(Some("Graphics"))));
    notebook.append_page(&form.sound_page, Some(&Label::new(Some("Sound"))));
    notebook.append_page(&form.input_page, Some(&Label::new(Some("Input"))));
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

/// Application tab: grouped settings — the DOSBox binary path, and (for the
/// AppImage) the desktop-shortcut actions.
fn build_app_tab(settings: &AppSettings, window: &Window) -> (GtkBox, Entry) {
    let page = widgets::page();

    // DOSBox section.
    let current = settings
        .dosbox_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let (row, dosbox_path, browse) = widgets::file_row("DOSBox binary", &current);
    page.append(&section(
        "DOSBox",
        "Leave empty to auto-detect dosbox-staging / dosbox on your PATH.",
        &row,
    ));
    wire_browse(window, &dosbox_path, &browse);

    // Desktop-shortcut section (AppImage only).
    if crate::integration::is_appimage() {
        let add = Button::with_label("Add to applications menu & desktop");
        add.add_css_class("suggested-action");
        let remove = Button::with_label("Remove from menu & desktop");
        remove.add_css_class("destructive-action");
        remove.set_sensitive(crate::integration::is_installed());

        // Stack the buttons, same width.
        let sizes = SizeGroup::new(SizeGroupMode::Horizontal);
        sizes.add_widget(&add);
        sizes.add_widget(&remove);
        let buttons = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .halign(gtk::Align::Start)
            .build();
        buttons.append(&add);
        buttons.append(&remove);

        page.append(&section(
            "Desktop shortcuts",
            "Add dosui to your applications menu and desktop so you can launch \
             it without locating the AppImage file.",
            &buttons,
        ));

        {
            let win = window.clone();
            let remove = remove.clone();
            add.connect_clicked(move |_| {
                crate::ui::desktop_integration::integrate_now(&win);
                remove.set_sensitive(true);
            });
        }
        {
            let win = window.clone();
            let remove2 = remove.clone();
            remove.connect_clicked(move |_| {
                crate::ui::desktop_integration::uninstall_now(&win);
                remove2.set_sensitive(false);
            });
        }
    }

    (page, dosbox_path)
}

/// A settings group: a heading, a dim description, and `content` in a padded
/// frame. Keeps the tab visually structured instead of a loose stack.
fn section(title: &str, description: &str, content: &impl IsA<Widget>) -> GtkBox {
    let group = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .margin_top(6)
        .build();
    group.append(
        &Label::builder()
            .label(title)
            .halign(gtk::Align::Start)
            .css_classes(["heading"])
            .build(),
    );
    group.append(
        &Label::builder()
            .label(description)
            .halign(gtk::Align::Start)
            .wrap(true)
            .css_classes(["dim-label"])
            .build(),
    );
    content.set_margin_top(10);
    content.set_margin_bottom(10);
    content.set_margin_start(12);
    content.set_margin_end(12);
    let frame = Frame::new(None);
    frame.set_child(Some(content));
    group.append(&frame);
    group
}

/// Wire the DOSBox-path "Browse…" button to a file chooser.
fn wire_browse(window: &Window, entry: &Entry, browse: &Button) {
    let window = window.downgrade();
    let entry = entry.clone();
    browse.connect_clicked(move |_| {
        let dialog = FileDialog::builder().title("Select DOSBox binary").build();
        let entry = entry.clone();
        dialog.open(
            window.upgrade().as_ref(),
            gio::Cancellable::NONE,
            move |res| {
                if let Ok(Some(path)) = res.map(|f| f.path()) {
                    entry.set_text(&path.display().to_string());
                }
            },
        );
    });
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
