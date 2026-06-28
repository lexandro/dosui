//! New-profile wizard: a 3-step guided alternative to the full editor.
//!
//! Steps: choose the game folder → pick the program to run (auto-scanned) →
//! name it. Finish builds a [`Profile`] (mount the folder as C:, run the chosen
//! program) and saves it under a fresh slug directory, then `on_created` refreshes
//! the list. Fine-tuning afterwards is done with the normal editor.

use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk::gio;
use gtk::prelude::*;
use gtk::{
    AlertDialog, ApplicationWindow, Box as GtkBox, Button, DropDown, Entry, FileDialog, Label,
    Orientation, Stack, StringList, Window,
};

use crate::config::dosbox_conf::DosboxConfig;
use crate::config::profile::{self, Mount, MountKind, Profile, RunSpec};
use crate::ui::widgets;

/// Stack page names, in wizard order.
const PAGES: [&str; 3] = ["folder", "program", "details"];

/// Wizard input widgets read on Finish.
#[derive(Clone)]
struct Wiz {
    folder: Entry,
    program: DropDown,
    title: Entry,
    genre: Entry,
    year: Entry,
}

pub fn open(parent: &ApplicationWindow, on_created: Rc<dyn Fn()>) {
    let window = Window::builder()
        .transient_for(parent)
        .modal(true)
        .title("New profile")
        .default_width(560)
        .default_height(430)
        .build();

    let stack = Stack::builder().vexpand(true).build();
    let (folder_page, folder, browse) = build_folder_page();
    let (program_page, program) = build_program_page();
    let (details_page, title, genre, year) = build_details_page();
    stack.add_named(&folder_page, Some("folder"));
    stack.add_named(&program_page, Some("program"));
    stack.add_named(&details_page, Some("details"));

    let wiz = Wiz {
        folder,
        program,
        title,
        genre,
        year,
    };

    let cancel = Button::with_label("Cancel");
    let back = Button::with_label("Back");
    let next = Button::builder()
        .label("Next")
        .css_classes(["suggested-action"])
        .build();
    let nav = nav_bar(&cancel, &back, &next);

    let outer = GtkBox::builder().orientation(Orientation::Vertical).build();
    outer.append(&stack);
    outer.append(&nav);
    window.set_child(Some(&outer));

    // Wire the folder browse button (needs the window as dialog parent).
    {
        let window = window.downgrade();
        let folder = wiz.folder.clone();
        browse.connect_clicked(move |_| pick_folder(&window, &folder));
    }

    let idx = Rc::new(Cell::new(0usize));
    let refresh_nav = {
        let stack = stack.clone();
        let back = back.clone();
        let next = next.clone();
        let idx = idx.clone();
        let wiz = wiz.clone();
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
            let ready = PAGES[i] != "folder" || !wiz.folder.text().trim().is_empty();
            next.set_sensitive(ready);
        })
    };
    refresh_nav();

    window.set_default_widget(Some(&next)); // Enter advances / creates
    wiz.folder.grab_focus();

    // Re-evaluate the Next button as the folder is typed or picked.
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
                on_enter_page(&wiz, i + 1);
                idx.set(i + 1);
                refresh_nav();
            } else {
                finish(&window, &wiz, &on_created);
            }
        });
    }

    // Expose folder/navigation as app actions while the wizard is open, so the
    // whole flow is driveable from outside (see dosui-ui-testing-via-gactions).
    if let Some(app) = parent.application() {
        register_wizard_actions(&app, &window, &wiz.folder, &back, &next, &cancel);
    }

    window.present();
}

/// Temporary app actions mirroring the wizard controls (`wizard-set-folder`
/// with a string path, `wizard-next` / `wizard-back` / `wizard-cancel`), removed
/// when the wizard closes. Lets the flow be scripted without on-screen input.
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

/// Prepare a page as it becomes visible (rescan executables, prefill the title).
fn on_enter_page(wiz: &Wiz, index: usize) {
    match PAGES[index] {
        "program" => {
            let exes = profile::scan_executables(Path::new(&wiz.folder.text().to_string()));
            let refs: Vec<&str> = exes.iter().map(String::as_str).collect();
            wiz.program.set_model(Some(&StringList::new(&refs)));
        }
        "details" if wiz.title.text().trim().is_empty() => {
            wiz.title.set_text(&default_title(&wiz.folder.text()));
        }
        _ => {}
    }
}

/// Build the final profile from the wizard and save it under a fresh directory.
fn finish(window: &Window, wiz: &Wiz, on_created: &Rc<dyn Fn()>) {
    let profile = build_profile(wiz);
    match save_new(profile) {
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

fn build_profile(wiz: &Wiz) -> Profile {
    let folder = wiz.folder.text().trim().to_string();
    let mounts = if folder.is_empty() {
        Vec::new()
    } else {
        vec![Mount {
            drive: 'C',
            kind: MountKind::Directory,
            path: PathBuf::from(folder),
            label: None,
        }]
    };
    let title = {
        let t = wiz.title.text().trim().to_string();
        if t.is_empty() {
            "New game".to_string()
        } else {
            t
        }
    };
    Profile {
        id: String::new(),
        title,
        genre: none_if_empty(&wiz.genre.text()),
        year: none_if_empty(&wiz.year.text()).and_then(|s| s.parse().ok()),
        developer: None,
        publisher: None,
        www: None,
        notes: None,
        cover: None,
        favorite: false,
        run: RunSpec {
            mounts,
            working_drive: 'C',
            command: widgets::dropdown_selected(&wiz.program).unwrap_or_default(),
            args: Vec::new(),
            exit_after: true,
        },
        dosbox: DosboxConfig::default(),
    }
}

fn save_new(mut profile: Profile) -> anyhow::Result<()> {
    let (id, dir) = profile::new_profile_dir(&profile.title)?;
    profile.id = id;
    profile.save(&dir)
}

fn build_folder_page() -> (GtkBox, Entry, Button) {
    let page = widgets::page();
    page.append(&heading("Step 1 — choose the game folder"));
    page.append(
        &Label::builder()
            .label("This folder is mounted as drive C:.")
            .halign(gtk::Align::Start)
            .css_classes(["dim-label"])
            .build(),
    );
    let (row, folder, browse) = widgets::file_row("Folder", "");
    page.append(&row);
    (page, folder, browse)
}

fn build_program_page() -> (GtkBox, DropDown) {
    let page = widgets::page();
    page.append(&heading("Step 2 — pick the program to run"));
    page.append(
        &Label::builder()
            .label("Executables found in the folder (.exe / .bat / .com).")
            .halign(gtk::Align::Start)
            .css_classes(["dim-label"])
            .build(),
    );
    let (row, program) = widgets::dropdown_row("Program", &[], None);
    page.append(&row);
    (page, program)
}

fn build_details_page() -> (GtkBox, Entry, Entry, Entry) {
    let page = widgets::page();
    page.append(&heading("Step 3 — name it"));
    let (row, title) = widgets::entry_row("Title", "");
    page.append(&row);
    let (row, genre) = widgets::entry_row("Genre", "");
    page.append(&row);
    let (row, year) = widgets::entry_row("Year", "");
    page.append(&row);
    (page, title, genre, year)
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
fn pick_folder(window: &gtk::glib::WeakRef<Window>, entry: &Entry) {
    let dialog = FileDialog::builder().title("Choose game folder").build();
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

fn heading(text: &str) -> Label {
    Label::builder()
        .label(text)
        .halign(gtk::Align::Start)
        .css_classes(["title-4"])
        .build()
}

/// Suggest a title from the folder's last path component.
fn default_title(folder: &str) -> String {
    Path::new(folder)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("New game")
        .to_string()
}

fn none_if_empty(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
