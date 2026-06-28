//! The profile editor's "Mounts & Run" tab: the run command plus a dynamic list
//! of drive mounts.
//!
//! Slightly over the 150-line soft cap: the dynamic add/remove mount-row
//! machinery is kept together with the tab it serves.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk::gio;
use gtk::glib::WeakRef;
use gtk::prelude::*;
use gtk::{
    Box as GtkBox, Button, CheckButton, DropDown, Entry, Label, Orientation, ScrolledWindow, Window,
};

use crate::config::profile::{Mount, MountKind, Profile, RunSpec};
use crate::ui::widgets;

const KIND_OPTIONS: [&str; 4] = ["Directory", "CD image", "Floppy image", "HDD image"];

/// Run-tab input widgets.
pub(crate) struct Run {
    working_drive: DropDown,
    command: Entry,
    args: Entry,
    exit_after: CheckButton,
    mounts: Rc<RefCell<Vec<MountRow>>>,
}

/// One mount entry's widgets, kept so we can read it back and remove it.
struct MountRow {
    container: GtkBox,
    drive: DropDown,
    kind: DropDown,
    path: Entry,
    label: Entry,
}

/// Build the Mounts & Run tab page and its widgets.
pub(crate) fn build(profile: &Profile, window: &Window) -> (GtkBox, Run) {
    let page = widgets::page();

    let (row, working_drive) = widgets::dropdown_row(
        "Working drive",
        &drive_letters(),
        Some(&profile.run.working_drive.to_string()),
    );
    page.append(&row);
    let (row, command) = widgets::entry_row("Command", &profile.run.command);
    page.append(&row);
    let (row, args) = widgets::entry_row("Arguments", &profile.run.args.join(" "));
    page.append(&row);
    let (row, exit_after) =
        widgets::check_row("Exit DOSBox when the program quits", profile.run.exit_after);
    page.append(&row);

    page.append(
        &Label::builder()
            .label("Mounts")
            .halign(gtk::Align::Start)
            .build(),
    );
    let mounts_box = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .build();
    page.append(
        &ScrolledWindow::builder()
            .child(&mounts_box)
            .min_content_height(140)
            .vexpand(true)
            .build(),
    );

    let mounts: Rc<RefCell<Vec<MountRow>>> = Rc::new(RefCell::new(Vec::new()));
    for mount in &profile.run.mounts {
        add_mount_row(window, &mounts_box, &mounts, Some(mount));
    }

    let add = Button::builder()
        .label("Add mount")
        .halign(gtk::Align::Start)
        .build();
    {
        let window = window.clone();
        let mounts_box = mounts_box.clone();
        let mounts = mounts.clone();
        add.connect_clicked(move |_| add_mount_row(&window, &mounts_box, &mounts, None));
    }
    page.append(&add);

    let widgets = Run {
        working_drive,
        command,
        args,
        exit_after,
        mounts,
    };
    (page, widgets)
}

impl Run {
    /// Apply the run spec onto `p`.
    pub(crate) fn apply(&self, p: &mut Profile) {
        p.run = RunSpec {
            working_drive: first_char(&widgets::dropdown_selected(&self.working_drive), 'C'),
            command: self.command.text().trim().to_string(),
            args: self
                .args
                .text()
                .split_whitespace()
                .map(str::to_string)
                .collect(),
            exit_after: self.exit_after.is_active(),
            mounts: collect_mounts(&self.mounts.borrow()),
        };
    }
}

/// Append one mount row (pre-filled from `mount` when editing) and track it.
fn add_mount_row(
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

fn collect_mounts(rows: &[MountRow]) -> Vec<Mount> {
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

/// Drive-letter options (A–Z), static so the refs are valid at build + read-back.
fn drive_letters() -> Vec<&'static str> {
    const L: [&str; 26] = [
        "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R",
        "S", "T", "U", "V", "W", "X", "Y", "Z",
    ];
    L.to_vec()
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

fn first_char(text: &Option<String>, fallback: char) -> char {
    text.as_deref()
        .and_then(|s| s.chars().next())
        .unwrap_or(fallback)
}
