//! Modal profile editor (GtkNotebook with one tab per concern).
//!
//! The editor loads a full [`Profile`], lets the user edit the fields the tabs
//! expose, and on Save writes `profile.toml` back. Tabs are added across M2; any
//! field a tab does not expose is preserved from the loaded profile (see
//! [`collect`]). `on_saved` lets the caller refresh the main list.

use std::path::PathBuf;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{
    AlertDialog, ApplicationWindow, Box as GtkBox, Button, Label, Notebook, Orientation,
    ScrolledWindow, TextView, Window,
};

use crate::config::profile::Profile;
use crate::ui::widgets;

/// Input widgets whose values are read back on Save.
struct Fields {
    title: gtk::Entry,
    genre: gtk::Entry,
    year: gtk::Entry,
    developer: gtk::Entry,
    publisher: gtk::Entry,
    www: gtk::Entry,
    notes: TextView,
}

/// Open the editor for an existing profile stored in `dir`.
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
        .default_width(560)
        .default_height(540)
        .build();

    let notebook = Notebook::new();
    notebook.set_vexpand(true);
    let (general, fields) = build_general(&profile);
    notebook.append_page(&general, Some(&Label::new(Some("General"))));

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

    {
        let window = window.clone();
        cancel.connect_clicked(move |_| window.close());
    }
    {
        let window = window.clone();
        let fields = Rc::new(fields);
        let original = profile;
        save.connect_clicked(move |_| {
            let updated = collect(&fields, &original);
            match updated.save(&dir) {
                Ok(()) => {
                    on_saved();
                    window.close();
                }
                Err(e) => {
                    log::error!("saving profile failed: {e:#}");
                    AlertDialog::builder()
                        .message("Could not save profile")
                        .detail(format!("{e:#}"))
                        .build()
                        .show(Some(&window));
                }
            }
        });
    }

    window.present();
}

/// Build the General tab and return its input widgets.
fn build_general(profile: &Profile) -> (GtkBox, Fields) {
    let page = widgets::page();

    let (row, title) = widgets::entry_row("Title", &profile.title);
    page.append(&row);
    let (row, genre) = widgets::entry_row("Genre", opt(&profile.genre));
    page.append(&row);
    let (row, year) = widgets::entry_row(
        "Year",
        &profile.year.map(|y| y.to_string()).unwrap_or_default(),
    );
    page.append(&row);
    let (row, developer) = widgets::entry_row("Developer", opt(&profile.developer));
    page.append(&row);
    let (row, publisher) = widgets::entry_row("Publisher", opt(&profile.publisher));
    page.append(&row);
    let (row, www) = widgets::entry_row("Website", opt(&profile.www));
    page.append(&row);

    page.append(
        &Label::builder()
            .label("Notes")
            .halign(gtk::Align::Start)
            .build(),
    );
    let notes = TextView::new();
    notes
        .buffer()
        .set_text(profile.notes.as_deref().unwrap_or(""));
    notes.set_wrap_mode(gtk::WrapMode::WordChar);
    let notes_scroller = ScrolledWindow::builder()
        .child(&notes)
        .min_content_height(110)
        .vexpand(true)
        .build();
    page.append(&notes_scroller);

    let fields = Fields {
        title,
        genre,
        year,
        developer,
        publisher,
        www,
        notes,
    };
    (page, fields)
}

/// Read the editor widgets into a profile, preserving fields not yet edited
/// (run spec, dosbox config) from `original`.
fn collect(fields: &Fields, original: &Profile) -> Profile {
    let mut p = original.clone();

    let title = fields.title.text().trim().to_string();
    if !title.is_empty() {
        p.title = title; // title is required; keep the old one if cleared
    }
    p.genre = none_if_empty(&fields.genre.text());
    p.year = none_if_empty(&fields.year.text()).and_then(|s| s.parse().ok());
    p.developer = none_if_empty(&fields.developer.text());
    p.publisher = none_if_empty(&fields.publisher.text());
    p.www = none_if_empty(&fields.www.text());

    let buffer = fields.notes.buffer();
    let notes = buffer
        .text(&buffer.start_iter(), &buffer.end_iter(), false)
        .to_string();
    p.notes = none_if_empty(&notes);

    p
}

/// `&str` view of an optional string for pre-filling an entry.
fn opt(value: &Option<String>) -> &str {
    value.as_deref().unwrap_or("")
}

/// `Some(trimmed)` unless the trimmed text is empty.
fn none_if_empty(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
