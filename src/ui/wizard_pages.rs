//! The new-profile wizard's pages and the profile it builds. The flow/navigation
//! lives in `wizard`; this module owns the page widgets ([`Wiz`]) and turns them
//! into a saved [`Profile`].

use std::path::{Path, PathBuf};

use gtk::prelude::*;
use gtk::{Button, DropDown, Entry, Label, Stack, StringList};

use crate::config::dosbox_conf::DosboxConfig;
use crate::config::profile::{self, Mount, MountKind, Profile, RunSpec};
use crate::ui::widgets;

/// Stack page names, in wizard order.
pub(crate) const PAGES: [&str; 3] = ["folder", "program", "details"];

/// Wizard input widgets read on Finish (plus the folder Browse button to wire).
#[derive(Clone)]
pub(crate) struct Wiz {
    pub folder: Entry,
    pub browse: Button,
    pub program: DropDown,
    pub title: Entry,
    pub genre: Entry,
    pub year: Entry,
}

/// Build the 3 pages into a Stack and return it with the input widgets.
pub(crate) fn build_stack() -> (Stack, Wiz) {
    let stack = Stack::builder().vexpand(true).build();

    let folder_page = widgets::page();
    folder_page.append(&heading("Step 1 — choose the game folder"));
    folder_page.append(&hint("This folder is mounted as drive C:."));
    let (row, folder, browse) = widgets::file_row("Folder", "");
    folder_page.append(&row);
    stack.add_named(&folder_page, Some("folder"));

    let program_page = widgets::page();
    program_page.append(&heading("Step 2 — pick the program to run"));
    program_page.append(&hint(
        "Executables found in the folder (.exe / .bat / .com).",
    ));
    let (row, program) = widgets::dropdown_row("Program", &[], None);
    program_page.append(&row);
    stack.add_named(&program_page, Some("program"));

    let details_page = widgets::page();
    details_page.append(&heading("Step 3 — name it"));
    let (row, title) = widgets::entry_row("Title", "");
    details_page.append(&row);
    let (row, genre) = widgets::entry_row("Genre", "");
    details_page.append(&row);
    let (row, year) = widgets::entry_row("Year", "");
    details_page.append(&row);
    stack.add_named(&details_page, Some("details"));

    let wiz = Wiz {
        folder,
        browse,
        program,
        title,
        genre,
        year,
    };
    (stack, wiz)
}

/// Prepare a page as it becomes visible (rescan executables, prefill the title).
pub(crate) fn on_enter_page(wiz: &Wiz, index: usize) {
    match PAGES[index] {
        "program" => {
            let exes = profile::scan_executables(Path::new(&wiz.folder.text().to_string()));
            let refs: Vec<&str> = exes.iter().map(String::as_str).collect();
            wiz.program.set_model(Some(&StringList::new(&refs)));
        }
        "details" if wiz.title.text().trim().is_empty() => {
            wiz.title.set_text(&default_title(&wiz.folder.text()));
        }
        _ => {}
    }
}

/// Build a profile from the wizard widgets and save it under a fresh directory.
pub(crate) fn save_new(wiz: &Wiz) -> anyhow::Result<()> {
    let mut profile = build_profile(wiz);
    let (id, dir) = profile::new_profile_dir(&profile.title)?;
    profile.id = id;
    profile.save(&dir)
}

fn build_profile(wiz: &Wiz) -> Profile {
    let folder = wiz.folder.text().trim().to_string();
    let mounts = if folder.is_empty() {
        Vec::new()
    } else {
        vec![Mount {
            drive: 'C',
            kind: MountKind::Directory,
            path: PathBuf::from(folder),
            label: None,
        }]
    };
    let title = widgets::none_if_empty(&wiz.title.text()).unwrap_or_else(|| "New game".to_string());
    Profile {
        id: String::new(),
        title,
        genre: widgets::none_if_empty(&wiz.genre.text()),
        year: widgets::none_if_empty(&wiz.year.text()).and_then(|s| s.parse().ok()),
        developer: None,
        publisher: None,
        www: None,
        notes: None,
        cover: None,
        favorite: false,
        last_played: None,
        run: RunSpec {
            mounts,
            working_drive: 'C',
            command: widgets::dropdown_selected(&wiz.program).unwrap_or_default(),
            args: Vec::new(),
            exit_after: true,
        },
        dosbox: DosboxConfig::default(),
    }
}

/// Suggest a title from the folder's last path component.
fn default_title(folder: &str) -> String {
    Path::new(folder)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("New game")
        .to_string()
}

fn heading(text: &str) -> Label {
    Label::builder()
        .label(text)
        .halign(gtk::Align::Start)
        .css_classes(["title-4"])
        .build()
}

fn hint(text: &str) -> Label {
    Label::builder()
        .label(text)
        .halign(gtk::Align::Start)
        .css_classes(["dim-label"])
        .build()
}
