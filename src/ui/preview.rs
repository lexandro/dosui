//! The bottom preview pane: a D-Fend-style tabbed `Notebook` under the games
//! list. Tabs: Screenshots · Notes · Data folder (D-Fend also has Sounds /
//! Videos, which dosui omits until it captures media). Each tab is its own
//! module; this file just assembles them and fans the selection out to each.

use std::path::Path;

use gtk::prelude::*;
use gtk::{Box as GtkBox, Image, Label, Notebook, Orientation};

use crate::config::profile::Profile;
use crate::ui::{preview_data, preview_notes, preview_screenshots};

#[derive(Clone)]
pub(crate) struct Preview {
    pub container: Notebook,
    screenshots: preview_screenshots::ScreenshotsTab,
    notes: preview_notes::NotesTab,
    data: preview_data::DataTab,
}

pub(crate) fn build() -> Preview {
    let screenshots = preview_screenshots::build();
    let notes = preview_notes::build();
    let data = preview_data::build();

    let container = Notebook::builder().show_border(false).build();
    container.append_page(
        &screenshots.root,
        Some(&tab("Screenshots", "image-x-generic-symbolic")),
    );
    container.append_page(
        &notes.root,
        Some(&tab("Notes", "accessories-text-editor-symbolic")),
    );
    container.append_page(&data.root, Some(&tab("Data folder", "folder-symbolic")));

    Preview {
        container,
        screenshots,
        notes,
        data,
    }
}

impl Preview {
    /// Point every tab at the selected profile.
    pub(crate) fn show(&self, dir: &Path, profile: &Profile) {
        self.screenshots.show(dir, profile);
        self.notes.show(dir, profile);
        self.data.show(dir, profile);
    }

    /// Reset every tab to its empty state (nothing selected).
    pub(crate) fn clear(&self) {
        self.screenshots.clear();
        self.notes.clear();
        self.data.clear();
    }
}

/// An icon + text tab label, like D-Fend's bottom tabs.
fn tab(text: &str, icon: &str) -> GtkBox {
    let label = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();
    label.append(&Image::from_icon_name(icon));
    label.append(&Label::new(Some(text)));
    label
}
