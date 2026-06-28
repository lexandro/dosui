//! The right-hand detail pane (cover, metadata, Play/Edit) plus the per-profile
//! display helpers (cover path, favourite star, meta line) shared with the grid.

use std::path::{Path, PathBuf};

use gtk::prelude::*;
use gtk::{Box as GtkBox, Button, ContentFit, Label, Orientation, Picture};

use crate::config::profile::{self, Profile};

/// Widgets in the detail pane whose contents change with the selection.
#[derive(Clone)]
pub(crate) struct Detail {
    pub container: GtkBox,
    pub cover: Picture,
    pub title: Label,
    pub meta: Label,
    pub notes: Label,
    pub last_played: Label,
    pub play: Button,
    pub edit: Button,
}

/// Build the detail pane (empty state until a profile is selected).
pub(crate) fn build_detail() -> Detail {
    let cover = Picture::builder()
        .content_fit(ContentFit::Contain)
        .height_request(180)
        .hexpand(true)
        .build();
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
    let last_played = Label::builder()
        .halign(gtk::Align::Start)
        .css_classes(["dim-label"])
        .build();
    let play = Button::builder()
        .label("Play")
        .css_classes(["suggested-action"])
        .sensitive(false)
        .build();
    let edit = Button::builder().label("Edit").sensitive(false).build();
    let actions = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .halign(gtk::Align::Start)
        .build();
    actions.append(&play);
    actions.append(&edit);

    let container = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .build();
    for w in [
        cover.upcast_ref::<gtk::Widget>(),
        title.upcast_ref(),
        meta.upcast_ref(),
        notes.upcast_ref(),
        last_played.upcast_ref(),
        actions.upcast_ref(),
    ] {
        container.append(w);
    }

    let detail = Detail {
        container,
        cover,
        title,
        meta,
        notes,
        last_played,
        play,
        edit,
    };
    clear_detail(&detail);
    detail
}

/// Fill the detail pane from a profile and enable Play/Edit.
pub(crate) fn show_profile(detail: &Detail, dir: &Path, profile: &Profile) {
    detail.title.set_text(&display_title(profile));
    detail.meta.set_text(&meta_line(profile));
    detail
        .notes
        .set_text(profile.notes.as_deref().unwrap_or(""));
    detail.last_played.set_text(&last_played_line(profile));
    set_cover(detail, cover_path(dir, profile).as_deref());
    detail.play.set_sensitive(true);
    detail.edit.set_sensitive(true);
}

/// Reset the detail pane to the empty state.
pub(crate) fn clear_detail(detail: &Detail) {
    detail.title.set_text("Select a profile");
    detail.meta.set_text("");
    detail.notes.set_text("");
    detail.last_played.set_text("");
    set_cover(detail, None);
    detail.play.set_sensitive(false);
    detail.edit.set_sensitive(false);
}

/// Resolve a profile's cover to an absolute path (relative covers join `dir`).
pub(crate) fn cover_path(dir: &Path, profile: &Profile) -> Option<PathBuf> {
    profile.cover.as_ref().map(|c| {
        if c.is_absolute() {
            c.clone()
        } else {
            dir.join(c)
        }
    })
}

/// Title with a leading star for favorites.
pub(crate) fn display_title(profile: &Profile) -> String {
    if profile.favorite {
        format!("★ {}", profile.title)
    } else {
        profile.title.clone()
    }
}

/// Show the cover image (hidden when absent or missing on disk).
fn set_cover(detail: &Detail, path: Option<&Path>) {
    match path {
        Some(p) if p.exists() => {
            detail.cover.set_filename(p.to_str());
            detail.cover.set_visible(true);
        }
        _ => {
            detail.cover.set_filename(None::<&str>);
            detail.cover.set_visible(false);
        }
    }
}

/// "Last played: …" line, or "Never played".
fn last_played_line(profile: &Profile) -> String {
    match profile.last_played {
        Some(then) => format!(
            "Last played: {}",
            profile::humanize_since(profile::now_unix(), then)
        ),
        None => "Never played".to_string(),
    }
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
