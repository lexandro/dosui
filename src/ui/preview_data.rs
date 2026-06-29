//! The preview "Data folder" tab: lists the entries of the selected profile's
//! directory (its `dosbox.conf`, cover, `drive_c`, …) and an "Open folder"
//! button that reuses the `app.open-folder` action to launch the file manager.

use std::path::Path;

use gtk::prelude::*;
use gtk::{
    Box as GtkBox, Button, Image, Label, ListBox, Orientation, ScrolledWindow, SelectionMode,
};

use crate::config::profile::Profile;

#[derive(Clone)]
pub(crate) struct DataTab {
    pub root: GtkBox,
    list: ListBox,
}

pub(crate) fn build() -> DataTab {
    let open = Button::builder()
        .label("Open folder")
        .icon_name("folder-open-symbolic")
        .halign(gtk::Align::Start)
        .margin_top(6)
        .margin_start(6)
        .build();
    open.set_action_name(Some("app.open-folder"));

    let list = ListBox::builder()
        .selection_mode(SelectionMode::None)
        .css_classes(["boxed-list"])
        .build();
    let scroller = ScrolledWindow::builder().child(&list).vexpand(true).build();

    let root = GtkBox::builder().orientation(Orientation::Vertical).build();
    root.append(&open);
    root.append(&scroller);
    DataTab { root, list }
}

impl DataTab {
    /// List the profile directory's entries (folders first, then files).
    pub(crate) fn show(&self, dir: &Path, _profile: &Profile) {
        self.clear();
        for (name, is_dir) in entries(dir) {
            let icon = Image::from_icon_name(if is_dir {
                "folder-symbolic"
            } else {
                "text-x-generic-symbolic"
            });
            let label = Label::builder()
                .label(&name)
                .halign(gtk::Align::Start)
                .build();
            let row = GtkBox::builder()
                .orientation(Orientation::Horizontal)
                .spacing(8)
                .margin_top(4)
                .margin_bottom(4)
                .margin_start(8)
                .margin_end(8)
                .build();
            row.append(&icon);
            row.append(&label);
            self.list.append(&row);
        }
    }

    pub(crate) fn clear(&self) {
        while let Some(row) = self.list.row_at_index(0) {
            self.list.remove(&row);
        }
    }
}

/// `(name, is_dir)` for each entry in `dir`, directories first then files, each
/// group sorted case-insensitively. Empty when the directory cannot be read.
fn entries(dir: &Path) -> Vec<(String, bool)> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<(String, bool)> = read
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_str()?.to_string();
            Some((name, e.path().is_dir()))
        })
        .collect();
    out.sort_by(|(an, ad), (bn, bd)| {
        bd.cmp(ad)
            .then_with(|| an.to_lowercase().cmp(&bn.to_lowercase()))
    });
    out
}
