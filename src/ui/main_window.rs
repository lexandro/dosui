//! The main application window: a D-Fend-style three-pane shell.
//!
//! Layout (see plan §2.5):
//!   ┌───────────────┬───────────────────────────┐
//!   │ category tree │ profile list (top)         │
//!   │ (sidebar)     ├───────────────────────────┤
//!   │               │ detail / media (bottom)    │
//!   └───────────────┴───────────────────────────┘
//!
//! M0 lays down the shell with placeholders; M1 fills the list + Play action.

use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box as GtkBox, Button, HeaderBar, Label, Orientation, Paned,
    ScrolledWindow, SearchEntry,
};

use crate::app::APP_NAME;

pub fn build(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title(APP_NAME)
        .default_width(960)
        .default_height(620)
        .build();

    window.set_titlebar(Some(&build_header()));
    window.set_child(Some(&build_body()));

    window.present();
}

/// Header bar: "+ New", search, and "Settings".
fn build_header() -> HeaderBar {
    let header = HeaderBar::new();

    let new_button = Button::builder()
        .icon_name("list-add-symbolic")
        .tooltip_text("New profile")
        .build();
    header.pack_start(&new_button);

    let settings_button = Button::builder()
        .icon_name("emblem-system-symbolic")
        .tooltip_text("Settings / global defaults")
        .build();
    header.pack_end(&settings_button);

    let search = SearchEntry::builder()
        .placeholder_text("Search profiles…")
        .build();
    header.set_title_widget(Some(&search));

    header
}

/// Body: horizontal split (category sidebar | content), content itself split
/// vertically into the profile list and the detail/media pane.
fn build_body() -> Paned {
    let sidebar = placeholder("Categories", "Genre · Developer · Year · Favorites");

    let profile_list = placeholder("Profiles", "Game list goes here (M1)");
    let detail = placeholder("Details", "Cover · metadata · Play (M1)");

    let content = Paned::builder()
        .orientation(Orientation::Vertical)
        .position(380)
        .build();
    content.set_start_child(Some(&profile_list));
    content.set_end_child(Some(&detail));

    let root = Paned::builder()
        .orientation(Orientation::Horizontal)
        .position(220)
        .build();
    root.set_start_child(Some(&sidebar));
    root.set_end_child(Some(&content));

    root
}

/// A simple bordered placeholder panel with a heading and a hint line.
fn placeholder(title: &str, hint: &str) -> ScrolledWindow {
    let heading = Label::builder()
        .label(title)
        .halign(gtk::Align::Start)
        .css_classes(["heading"])
        .build();

    let hint_label = Label::builder()
        .label(hint)
        .halign(gtk::Align::Start)
        .css_classes(["dim-label"])
        .wrap(true)
        .build();

    let vbox = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    vbox.append(&heading);
    vbox.append(&hint_label);

    ScrolledWindow::builder().child(&vbox).build()
}
