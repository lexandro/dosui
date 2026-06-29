//! The preview "Screenshots" tab: a thumbnail wall of the profile's cover plus
//! any image files found in its directory (D-Fend shows captured screenshots
//! here; dosui has no capture pipeline yet, so it shows whatever images live in
//! the profile folder). Empty profiles get a dim placeholder.

use std::path::Path;

use gtk::prelude::*;
use gtk::{
    Box as GtkBox, ContentFit, FlowBox, Label, Orientation, Picture, PolicyType, ScrolledWindow,
    SelectionMode,
};

use crate::config::console;
use crate::config::profile::Profile;
use crate::ui::display::{console_paintable, cover_path};

/// Image file extensions surfaced as screenshots.
const IMAGE_EXTS: [&str; 5] = ["png", "jpg", "jpeg", "gif", "bmp"];

#[derive(Clone)]
pub(crate) struct ScreenshotsTab {
    pub root: ScrolledWindow,
    flow: FlowBox,
}

pub(crate) fn build() -> ScreenshotsTab {
    let flow = FlowBox::builder()
        .orientation(Orientation::Horizontal)
        .selection_mode(SelectionMode::None)
        .homogeneous(true)
        .row_spacing(8)
        .column_spacing(8)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .build();
    let root = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .child(&flow)
        .build();
    ScreenshotsTab { root, flow }
}

impl ScreenshotsTab {
    /// Show the cover and any images found in the profile directory.
    pub(crate) fn show(&self, dir: &Path, profile: &Profile) {
        self.clear();
        let mut shown = 0;
        if let Some(cover) = cover_path(dir, profile).filter(|p| p.exists()) {
            self.add_image(&cover);
            shown += 1;
        }
        for path in image_files(dir) {
            if cover_path(dir, profile).as_deref() == Some(path.as_path()) {
                continue; // already shown as the cover
            }
            self.add_image(&path);
            shown += 1;
        }
        if shown == 0 {
            self.add_placeholder(profile);
        }
    }

    pub(crate) fn clear(&self) {
        while let Some(child) = self.flow.first_child() {
            self.flow.remove(&child);
        }
    }

    /// Append one thumbnail tile (picture + file name).
    fn add_image(&self, path: &Path) {
        let picture = Picture::builder()
            .content_fit(ContentFit::Contain)
            .width_request(160)
            .height_request(120)
            .build();
        picture.set_filename(path.to_str());
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        self.flow.insert(&tile(picture.upcast(), &name), -1);
    }

    /// Empty state: the console icon for a bare prompt, else a dim message.
    fn add_placeholder(&self, profile: &Profile) {
        if console::is_console(profile) {
            let picture = Picture::builder()
                .content_fit(ContentFit::Contain)
                .width_request(160)
                .height_request(120)
                .build();
            picture.set_paintable(Some(&console_paintable(&picture)));
            self.flow.insert(&tile(picture.upcast(), "DOS Console"), -1);
        } else {
            let label = Label::builder()
                .label("No screenshots")
                .css_classes(["dim-label"])
                .margin_top(16)
                .build();
            self.flow.insert(&label, -1);
        }
    }
}

/// A vertical tile: the thumbnail widget over its file-name label.
fn tile(widget: gtk::Widget, name: &str) -> GtkBox {
    let label = Label::builder()
        .label(name)
        .ellipsize(gtk::pango::EllipsizeMode::Middle)
        .max_width_chars(22)
        .css_classes(["dim-label"])
        .build();
    let cell = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .build();
    cell.append(&widget);
    cell.append(&label);
    cell
}

/// Image files directly in `dir`, sorted case-insensitively by name.
fn image_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<std::path::PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && has_image_ext(p))
        .collect();
    out.sort_by_key(|p| p.to_string_lossy().to_lowercase());
    out
}

fn has_image_ext(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .is_some_and(|e| IMAGE_EXTS.contains(&e.as_str()))
}
