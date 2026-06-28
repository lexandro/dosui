//! The main application window: profile list (left) + detail/Play (right).
//!
//! M1 is a two-pane MVP: pick a profile, hit Play, DOSBox runs. The category
//! sidebar and cover grid arrive in later milestones (plan §2.5).

use std::path::PathBuf;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{
    AlertDialog, Application, ApplicationWindow, Box as GtkBox, Button, HeaderBar, Label, ListBox,
    Orientation, Paned, ScrolledWindow, SearchEntry, SelectionMode,
};

use crate::app::APP_NAME;
use crate::config::profile::{self, Profile};
use crate::config::settings::AppSettings;
use crate::launcher;

/// Loaded profile together with its on-disk directory (needed to launch).
type Entry = (PathBuf, Profile);

/// Widgets in the detail pane whose text changes with the selection.
#[derive(Clone)]
struct Detail {
    container: GtkBox,
    title: Label,
    meta: Label,
    notes: Label,
    play: Button,
}

pub fn build(app: &Application) {
    let profiles: Rc<Vec<Entry>> = Rc::new(load_profiles());
    let settings = Rc::new(AppSettings::load());

    let window = ApplicationWindow::builder()
        .application(app)
        .title(APP_NAME)
        .default_width(900)
        .default_height(580)
        .build();
    window.set_titlebar(Some(&build_header()));

    let list = ListBox::new();
    list.set_selection_mode(SelectionMode::Single);
    for (_, profile) in profiles.iter() {
        list.append(&profile_row(profile));
    }
    let list_scroller = ScrolledWindow::builder()
        .child(&list)
        .width_request(260)
        .build();

    let detail = build_detail();

    // Selection -> refresh the detail pane.
    {
        let profiles = profiles.clone();
        let detail = detail.clone();
        list.connect_row_selected(move |_, row| match row {
            Some(row) => {
                if let Some((_, profile)) = profiles.get(row.index() as usize) {
                    show_profile(&detail, profile);
                }
            }
            None => clear_detail(&detail),
        });
    }

    // Play -> launch the selected profile.
    {
        let profiles = profiles.clone();
        let settings = settings.clone();
        let window = window.downgrade(); // weak: avoid window<->closure cycle
        let list = list.clone();
        detail.play.connect_clicked(move |_| {
            let Some(row) = list.selected_row() else {
                return;
            };
            let Some((dir, profile)) = profiles.get(row.index() as usize) else {
                return;
            };
            if let Err(e) = launcher::launch(dir, profile, &settings) {
                log::error!("launch failed: {e:#}");
                if let Some(window) = window.upgrade() {
                    AlertDialog::builder()
                        .message(format!("Could not launch {}", profile.title))
                        .detail(format!("{e:#}"))
                        .build()
                        .show(Some(&window));
                }
            }
        });
    }

    let root = Paned::builder()
        .orientation(Orientation::Horizontal)
        .position(260)
        .start_child(&list_scroller)
        .end_child(&detail.container)
        .build();
    window.set_child(Some(&root));

    window.present();
}

/// Load every profile from the data dir; an error yields an empty list (logged).
fn load_profiles() -> Vec<Entry> {
    match crate::config::paths::profiles_dir().and_then(|dir| profile::scan(&dir)) {
        Ok(entries) => entries,
        Err(e) => {
            log::error!("loading profiles: {e:#}");
            Vec::new()
        }
    }
}

fn build_header() -> HeaderBar {
    let header = HeaderBar::new();
    header.pack_start(
        &Button::builder()
            .icon_name("list-add-symbolic")
            .tooltip_text("New profile (M3)")
            .build(),
    );
    header.pack_end(
        &Button::builder()
            .icon_name("emblem-system-symbolic")
            .tooltip_text("Settings (M4)")
            .build(),
    );
    header.set_title_widget(Some(
        &SearchEntry::builder()
            .placeholder_text("Search profiles…")
            .build(),
    ));
    header
}

/// One list row: the profile title.
fn profile_row(profile: &Profile) -> Label {
    Label::builder()
        .label(&profile.title)
        .halign(gtk::Align::Start)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(10)
        .margin_end(10)
        .build()
}

/// Build the detail pane (empty state until a profile is selected).
fn build_detail() -> Detail {
    let title = Label::builder()
        .halign(gtk::Align::Start)
        .css_classes(["title-2"])
        .build();
    let meta = Label::builder()
        .halign(gtk::Align::Start)
        .css_classes(["dim-label"])
        .build();
    let notes = Label::builder()
        .halign(gtk::Align::Start)
        .wrap(true)
        .build();
    let play = Button::builder()
        .label("Play")
        .css_classes(["suggested-action"])
        .halign(gtk::Align::Start)
        .sensitive(false)
        .build();

    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .build();
    container.append(&title);
    container.append(&meta);
    container.append(&notes);
    container.append(&play);

    let detail = Detail {
        container,
        title,
        meta,
        notes,
        play,
    };
    clear_detail(&detail);
    detail
}

/// Fill the detail pane from a profile and enable Play.
fn show_profile(detail: &Detail, profile: &Profile) {
    detail.title.set_text(&profile.title);
    detail.meta.set_text(&meta_line(profile));
    detail
        .notes
        .set_text(profile.notes.as_deref().unwrap_or(""));
    detail.play.set_sensitive(true);
}

/// Reset the detail pane to the empty state.
fn clear_detail(detail: &Detail) {
    detail.title.set_text("Select a profile");
    detail.meta.set_text("");
    detail.notes.set_text("");
    detail.play.set_sensitive(false);
}

/// "Genre · Year · Developer" from whatever fields are present.
fn meta_line(profile: &Profile) -> String {
    let mut parts = Vec::new();
    if let Some(genre) = &profile.genre {
        parts.push(genre.clone());
    }
    if let Some(year) = profile.year {
        parts.push(year.to_string());
    }
    if let Some(dev) = &profile.developer {
        parts.push(dev.clone());
    }
    parts.join(" · ")
}
