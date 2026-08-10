//! One mount entry: a drive/kind/path/label row with Browse and remove buttons,
//! created dynamically and read back into [`Mount`]s.
//!
//! Marginally over the 150-line soft cap: one row widget plus the read-back
//! that interprets its fields.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk::gio;
use gtk::glib::WeakRef;
use gtk::prelude::*;
use gtk::{Box as GtkBox, Button, DropDown, Entry, Orientation, Window};

use super::{drive_letters, first_char};
use crate::config::profile::{Mount, MountKind};
use crate::ui::widgets;

const KIND_OPTIONS: [&str; 4] = ["Directory", "CD image", "Floppy image", "HDD image"];

/// One mount entry's widgets, kept so we can read it back and remove it.
pub(super) struct MountRow {
    container: GtkBox,
    drive: DropDown,
    kind: DropDown,
    path: Entry,
    label: Entry,
}

/// Append one mount row (pre-filled from `mount` when editing) and track it.
pub(super) fn add_row(
    window: &Window,
    mounts_box: &GtkBox,
    mounts: &Rc<RefCell<Vec<MountRow>>>,
    mount: Option<&Mount>,
) {
    let drive = widgets::dropdown(
        &drive_letters(),
        Some(&mount.map(|m| m.drive).unwrap_or('C').to_string()),
    );
    let kind = widgets::dropdown(
        &KIND_OPTIONS,
        Some(kind_to_text(
            mount.map(|m| m.kind).unwrap_or(MountKind::Directory),
        )),
    );
    let path = Entry::builder()
        .text(
            mount
                .map(|m| m.path.display().to_string())
                .unwrap_or_default(),
        )
        .hexpand(true)
        .build();
    let browse = Button::with_label("Browse…");
    let label = Entry::builder()
        .placeholder_text("label (optional)")
        .text(mount.and_then(|m| m.label.clone()).unwrap_or_default())
        .width_request(120)
        .build();
    let remove = Button::from_icon_name("list-remove-symbolic");

    let container = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();
    for w in [
        drive.upcast_ref::<gtk::Widget>(),
        kind.upcast_ref(),
        path.upcast_ref(),
        browse.upcast_ref(),
        label.upcast_ref(),
        remove.upcast_ref(),
    ] {
        container.append(w);
    }
    mounts_box.append(&container);

    {
        let window = window.downgrade();
        let path = path.clone();
        let kind = kind.clone();
        browse.connect_clicked(move |_| pick_path(&window, &path, &kind));
    }
    {
        let mounts_box = mounts_box.clone();
        let mounts = mounts.clone();
        let container = container.clone();
        remove.connect_clicked(move |_| {
            mounts_box.remove(&container);
            mounts.borrow_mut().retain(|r| r.container != container);
        });
    }

    mounts.borrow_mut().push(MountRow {
        container,
        drive,
        kind,
        path,
        label,
    });
}

/// Read the tracked rows into [`Mount`]s, dropping rows with an empty path.
pub(super) fn collect(rows: &[MountRow]) -> Vec<Mount> {
    rows.iter()
        .filter_map(|row| {
            let path = row.path.text().trim().to_string();
            if path.is_empty() {
                return None; // a mount with no path is meaningless; drop it
            }
            Some(Mount {
                drive: first_char(&widgets::dropdown_selected(&row.drive), 'C'),
                kind: text_to_kind(&widgets::dropdown_selected(&row.kind).unwrap_or_default()),
                path: PathBuf::from(path),
                label: widgets::none_if_empty(&row.label.text()),
            })
        })
        .collect()
}

/// Open a file/folder chooser and write the chosen path into `entry`.
fn pick_path(window: &WeakRef<Window>, entry: &Entry, kind: &DropDown) {
    let dialog = gtk::FileDialog::builder()
        .title("Select mount path")
        .build();
    let parent = window.upgrade();
    let entry = entry.clone();
    let is_dir = widgets::dropdown_selected(kind).as_deref() == Some("Directory");
    let on_done = move |res: Result<gio::File, gtk::glib::Error>| {
        if let Ok(file) = res {
            if let Some(path) = file.path() {
                entry.set_text(&path.display().to_string());
            }
        }
    };
    if is_dir {
        dialog.select_folder(parent.as_ref(), gio::Cancellable::NONE, on_done);
    } else {
        dialog.open(parent.as_ref(), gio::Cancellable::NONE, on_done);
    }
}

fn kind_to_text(kind: MountKind) -> &'static str {
    match kind {
        MountKind::Directory => "Directory",
        MountKind::CdImage => "CD image",
        MountKind::FloppyImage => "Floppy image",
        MountKind::HddImage => "HDD image",
    }
}

fn text_to_kind(text: &str) -> MountKind {
    match text {
        "CD image" => MountKind::CdImage,
        "Floppy image" => MountKind::FloppyImage,
        "HDD image" => MountKind::HddImage,
        _ => MountKind::Directory,
    }
}
