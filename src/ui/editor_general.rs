//! The profile editor's "General" tab: title, metadata, cover, and notes.

use gtk::gio;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Entry, FileDialog, Label, ScrolledWindow, TextView, Window};

use crate::config::profile::Profile;
use crate::ui::widgets;

/// General-tab input widgets.
pub(crate) struct General {
    title: Entry,
    genre: Entry,
    year: Entry,
    developer: Entry,
    publisher: Entry,
    www: Entry,
    cover: Entry,
    notes: TextView,
}

/// Build the General tab page and its widgets.
pub(crate) fn build(profile: &Profile, window: &Window) -> (GtkBox, General) {
    let page = widgets::page();

    let (row, title) = widgets::entry_row("Title", &profile.title);
    page.append(&row);
    let (row, genre) = widgets::entry_row("Genre", widgets::opt(&profile.genre));
    page.append(&row);
    let (row, year) = widgets::entry_row(
        "Year",
        &profile.year.map(|y| y.to_string()).unwrap_or_default(),
    );
    page.append(&row);
    let (row, developer) = widgets::entry_row("Developer", widgets::opt(&profile.developer));
    page.append(&row);
    let (row, publisher) = widgets::entry_row("Publisher", widgets::opt(&profile.publisher));
    page.append(&row);
    let (row, www) = widgets::entry_row("Website", widgets::opt(&profile.www));
    page.append(&row);

    let cover_text = profile
        .cover
        .as_ref()
        .map(|c| c.display().to_string())
        .unwrap_or_default();
    let (row, cover, browse) = widgets::file_row("Cover image", &cover_text);
    page.append(&row);
    wire_cover_browse(window, &cover, &browse);

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
    page.append(
        &ScrolledWindow::builder()
            .child(&notes)
            .min_content_height(110)
            .vexpand(true)
            .build(),
    );

    let widgets = General {
        title,
        genre,
        year,
        developer,
        publisher,
        www,
        cover,
        notes,
    };
    (page, widgets)
}

impl General {
    /// Apply the edited metadata onto `p` (title is kept if cleared).
    pub(crate) fn apply(&self, p: &mut Profile) {
        let title = self.title.text().trim().to_string();
        if !title.is_empty() {
            p.title = title;
        }
        p.genre = widgets::none_if_empty(&self.genre.text());
        p.year = widgets::none_if_empty(&self.year.text()).and_then(|s| s.parse().ok());
        p.developer = widgets::none_if_empty(&self.developer.text());
        p.publisher = widgets::none_if_empty(&self.publisher.text());
        p.www = widgets::none_if_empty(&self.www.text());
        p.cover = widgets::none_if_empty(&self.cover.text()).map(Into::into);
        p.notes = widgets::none_if_empty(&widgets::textview_text(&self.notes));
    }
}

/// Wire the cover "Browse…" button to an image picker.
fn wire_cover_browse(window: &Window, cover: &Entry, browse: &gtk::Button) {
    let window = window.downgrade();
    let cover = cover.clone();
    browse.connect_clicked(move |_| {
        let dialog = FileDialog::builder().title("Select cover image").build();
        let cover = cover.clone();
        dialog.open(
            window.upgrade().as_ref(),
            gio::Cancellable::NONE,
            move |res| {
                if let Ok(file) = res {
                    if let Some(path) = file.path() {
                        cover.set_text(&path.display().to_string());
                    }
                }
            },
        );
    });
}
