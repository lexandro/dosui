//! New-profile wizard: a 3-step guided alternative to the full editor (choose
//! folder → pick program → name it). This module is the flow/navigation; the
//! pages and the profile they build live in [`crate::ui::wizard_pages`].

use std::cell::Cell;
use std::rc::Rc;

use gtk::gio;
use gtk::glib::WeakRef;
use gtk::prelude::*;
use gtk::{AlertDialog, ApplicationWindow, Box as GtkBox, Button, Entry, Orientation, Window};

use crate::ui::wizard_pages::{self, Wiz, PAGES};

pub fn open(parent: &ApplicationWindow, on_created: Rc<dyn Fn()>) {
    let window = Window::builder()
        .transient_for(parent)
        .modal(true)
        .title("New profile")
        .default_width(560)
        .default_height(430)
        .build();

    let (stack, wiz) = wizard_pages::build_stack();

    let cancel = Button::with_label("Cancel");
    let back = Button::with_label("Back");
    let next = Button::builder()
        .label("Next")
        .css_classes(["suggested-action"])
        .build();

    let outer = GtkBox::builder().orientation(Orientation::Vertical).build();
    outer.append(&stack);
    outer.append(&nav_bar(&cancel, &back, &next));
    window.set_child(Some(&outer));
    window.set_default_widget(Some(&next)); // Enter advances / creates

    {
        let window = window.downgrade();
        let folder = wiz.folder.clone();
        wiz.browse
            .connect_clicked(move |_| pick_folder(&window, &folder));
    }

    let idx = Rc::new(Cell::new(0usize));
    let refresh_nav = {
        let stack = stack.clone();
        let back = back.clone();
        let next = next.clone();
        let idx = idx.clone();
        let folder = wiz.folder.clone();
        Rc::new(move || {
            let i = idx.get();
            stack.set_visible_child_name(PAGES[i]);
            back.set_sensitive(i > 0);
            next.set_label(if i == PAGES.len() - 1 {
                "Create"
            } else {
                "Next"
            });
            // Step 1 requires a folder; later steps are always proceedable.
            next.set_sensitive(PAGES[i] != "folder" || !folder.text().trim().is_empty());
        })
    };
    refresh_nav();
    wiz.folder.grab_focus();

    {
        let refresh_nav = refresh_nav.clone();
        wiz.folder.connect_changed(move |_| refresh_nav());
    }
    {
        let window = window.clone();
        cancel.connect_clicked(move |_| window.close());
    }
    {
        let idx = idx.clone();
        let refresh_nav = refresh_nav.clone();
        back.connect_clicked(move |_| {
            let i = idx.get();
            if i > 0 {
                idx.set(i - 1);
                refresh_nav();
            }
        });
    }
    {
        let idx = idx.clone();
        let refresh_nav = refresh_nav.clone();
        let window = window.clone();
        let wiz = wiz.clone();
        let on_created = on_created.clone();
        next.connect_clicked(move |_| {
            let i = idx.get();
            if i < PAGES.len() - 1 {
                wizard_pages::on_enter_page(&wiz, i + 1);
                idx.set(i + 1);
                refresh_nav();
            } else {
                finish(&window, &wiz, &on_created);
            }
        });
    }

    // Expose folder/navigation as app actions while open (see actions in tests).
    if let Some(app) = parent.application() {
        register_wizard_actions(&app, &window, &wiz.folder, &back, &next, &cancel);
    }

    window.present();
}

/// Build the profile and close, or report the error.
fn finish(window: &Window, wiz: &Wiz, on_created: &Rc<dyn Fn()>) {
    match wizard_pages::save_new(wiz) {
        Ok(()) => {
            on_created();
            window.close();
        }
        Err(e) => {
            log::error!("creating profile failed: {e:#}");
            AlertDialog::builder()
                .message("Could not create profile")
                .detail(format!("{e:#}"))
                .build()
                .show(Some(window));
        }
    }
}

fn nav_bar(cancel: &Button, back: &Button, next: &Button) -> GtkBox {
    let nav = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .build();
    nav.append(cancel);
    let spacer = GtkBox::builder().hexpand(true).build();
    nav.append(&spacer);
    nav.append(back);
    nav.append(next);
    nav
}

/// Open a folder chooser and write the chosen path into `entry`.
fn pick_folder(window: &WeakRef<Window>, entry: &Entry) {
    let dialog = gtk::FileDialog::builder()
        .title("Choose game folder")
        .build();
    let entry = entry.clone();
    dialog.select_folder(
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
}

/// Temporary app actions mirroring the controls (`wizard-set-folder` with a
/// string path, `wizard-next` / `wizard-back` / `wizard-cancel`), removed on close.
/// Lets the flow be scripted without on-screen input.
fn register_wizard_actions(
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
