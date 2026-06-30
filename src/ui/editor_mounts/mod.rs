//! The profile editor's "Mounts & Run" tab: the run command plus a dynamic list
//! of drive mounts. The add/remove mount-row machinery lives in [`mount_row`].

mod mount_row;

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{
    Box as GtkBox, Button, CheckButton, DropDown, Entry, Label, Orientation, ScrolledWindow, Window,
};

use crate::config::profile::{Profile, RunSpec};
use crate::ui::widgets;
use mount_row::MountRow;

/// Run-tab input widgets.
pub(crate) struct Run {
    working_drive: DropDown,
    command: Entry,
    args: Entry,
    exit_after: CheckButton,
    mounts: Rc<RefCell<Vec<MountRow>>>,
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
        mount_row::add_row(window, &mounts_box, &mounts, Some(mount));
    }

    let add = Button::builder()
        .label("Add mount")
        .halign(gtk::Align::Start)
        .build();
    {
        let window = window.clone();
        let mounts_box = mounts_box.clone();
        let mounts = mounts.clone();
        add.connect_clicked(move |_| mount_row::add_row(&window, &mounts_box, &mounts, None));
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
            mounts: mount_row::collect(&self.mounts.borrow()),
        };
    }
}

/// Drive-letter options (A–Z), static so the refs are valid at build + read-back.
pub(super) fn drive_letters() -> Vec<&'static str> {
    const L: [&str; 26] = [
        "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R",
        "S", "T", "U", "V", "W", "X", "Y", "Z",
    ];
    L.to_vec()
}

pub(super) fn first_char(text: &Option<String>, fallback: char) -> char {
    text.as_deref()
        .and_then(|s| s.chars().next())
        .unwrap_or(fallback)
}
