//! Bulk metadata editor: set genre / developer / publisher / year on many
//! profiles at once (D-Fend's bulk edit). A self-contained dialog so it doesn't
//! disturb the main grid's single-selection model.
//!
//! Each field has a "Set" toggle — only toggled fields are written, so blank
//! entries never accidentally clear data. Profiles are chosen via checkboxes.

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{
    AlertDialog, ApplicationWindow, Box as GtkBox, Button, CheckButton, Entry, Label, Orientation,
    ScrolledWindow, Separator, Window,
};

use crate::config::profile::Profile;

/// A "Set X" toggle paired with its value entry.
struct Field {
    set: CheckButton,
    entry: Entry,
}

impl Field {
    /// The new value if this field is enabled: `Some(Some(v))` to set, `Some(None)`
    /// to clear, `None` to leave untouched.
    fn change(&self) -> Option<Option<String>> {
        self.set
            .is_active()
            .then(|| none_if_empty(&self.entry.text()))
    }
}

/// Open the bulk editor for all `entries`. `on_done` runs after a successful apply.
pub fn open(parent: &ApplicationWindow, entries: Vec<(PathBuf, Profile)>, on_done: Rc<dyn Fn()>) {
    let window = Window::builder()
        .transient_for(parent)
        .modal(true)
        .title("Bulk edit")
        .default_width(460)
        .default_height(520)
        .build();

    let page = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(14)
        .margin_bottom(14)
        .margin_start(14)
        .margin_end(14)
        .build();

    page.append(&heading("Set these fields on the chosen profiles:"));
    let genre = field_row(&page, "Genre");
    let developer = field_row(&page, "Developer");
    let publisher = field_row(&page, "Publisher");
    let year = field_row(&page, "Year");
    let fields = Rc::new([genre, developer, publisher, year]); // genre, developer, publisher, year

    page.append(&Separator::new(Orientation::Horizontal));
    page.append(&heading("Apply to:"));

    let select_all = CheckButton::with_label("Select all");
    page.append(&select_all);

    let list = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .build();
    let checks: Rc<RefCell<Vec<CheckButton>>> = Rc::new(RefCell::new(Vec::new()));
    for (_, profile) in &entries {
        let check = CheckButton::with_label(&profile.title);
        list.append(&check);
        checks.borrow_mut().push(check);
    }
    page.append(&ScrolledWindow::builder().child(&list).vexpand(true).build());

    {
        let checks = checks.clone();
        select_all.connect_toggled(move |btn| {
            let on = btn.is_active();
            for c in checks.borrow().iter() {
                c.set_active(on);
            }
        });
    }

    let cancel = Button::with_label("Cancel");
    let apply = Button::builder()
        .label("Apply")
        .css_classes(["suggested-action"])
        .build();
    let actions = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .halign(gtk::Align::End)
        .build();
    actions.append(&cancel);
    actions.append(&apply);
    page.append(&actions);

    window.set_child(Some(&page));
    window.set_default_widget(Some(&apply));

    {
        let window = window.clone();
        cancel.connect_clicked(move |_| window.close());
    }
    {
        let window = window.clone();
        let entries = Rc::new(entries);
        apply.connect_clicked(
            move |_| match apply_changes(&entries, &checks.borrow(), &fields) {
                Ok(()) => {
                    on_done();
                    window.close();
                }
                Err(e) => {
                    log::error!("bulk edit failed: {e:#}");
                    AlertDialog::builder()
                        .message("Could not apply bulk edit")
                        .detail(format!("{e:#}"))
                        .build()
                        .show(Some(&window));
                }
            },
        );
    }

    window.present();
}

/// Write the enabled field changes to every checked profile.
fn apply_changes(
    entries: &[(PathBuf, Profile)],
    checks: &[CheckButton],
    fields: &[Field; 4],
) -> anyhow::Result<()> {
    for (i, (dir, profile)) in entries.iter().enumerate() {
        if !checks.get(i).is_some_and(CheckButton::is_active) {
            continue;
        }
        let mut p = profile.clone();
        if let Some(v) = fields[0].change() {
            p.genre = v;
        }
        if let Some(v) = fields[1].change() {
            p.developer = v;
        }
        if let Some(v) = fields[2].change() {
            p.publisher = v;
        }
        if let Some(v) = fields[3].change() {
            p.year = v.and_then(|s| s.parse().ok());
        }
        p.save(dir)?;
    }
    Ok(())
}

fn field_row(page: &GtkBox, label: &str) -> Field {
    let set = CheckButton::with_label(label);
    set.set_width_request(110);
    let entry = Entry::builder().hexpand(true).build();
    let row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    row.append(&set);
    row.append(&entry);
    page.append(&row);
    Field { set, entry }
}

fn heading(text: &str) -> Label {
    Label::builder()
        .label(text)
        .halign(gtk::Align::Start)
        .css_classes(["dim-label"])
        .build()
}

fn none_if_empty(text: &str) -> Option<String> {
    let t = text.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}
