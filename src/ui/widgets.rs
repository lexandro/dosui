//! Small, reusable form-row builders for the profile editor.
//!
//! Each `*_row` returns the row container plus the input widget(s) so the caller
//! can read values back on save. Keeping these here keeps the editor tabs terse
//! and visually consistent (aligned labels, uniform spacing).

use gtk::prelude::*;
use gtk::{
    Align, Box as GtkBox, Button, CheckButton, DropDown, Entry, Label, Orientation, TextView,
};

const LABEL_WIDTH: i32 = 150;

/// A vertical, padded container for one notebook tab.
pub fn page() -> GtkBox {
    GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(14)
        .margin_bottom(14)
        .margin_start(14)
        .margin_end(14)
        .build()
}

/// `label   [ control ]` on one line, with a fixed-width label for alignment.
fn labeled(label: &str, control: &impl IsA<gtk::Widget>) -> GtkBox {
    let row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    row.append(
        &Label::builder()
            .label(label)
            .halign(Align::Start)
            .width_request(LABEL_WIDTH)
            .build(),
    );
    control.set_hexpand(true);
    row.append(control);
    row
}

/// Text entry row.
pub fn entry_row(label: &str, value: &str) -> (GtkBox, Entry) {
    let entry = Entry::builder().text(value).hexpand(true).build();
    (labeled(label, &entry), entry)
}

/// Checkbox row (the box label sits to the right of the toggle).
pub fn check_row(label: &str, active: bool) -> (GtkBox, CheckButton) {
    let check = CheckButton::builder().label(label).active(active).build();
    let row = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .build();
    row.append(&check);
    (row, check)
}

/// File/folder picker row: a path entry plus a "Browse…" button. The caller
/// wires the button to a `FileDialog` (it needs the parent window).
pub fn file_row(label: &str, value: &str) -> (GtkBox, Entry, Button) {
    let entry = Entry::builder().text(value).hexpand(true).build();
    let browse = Button::with_label("Browse…");
    let inner = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();
    inner.append(&entry);
    inner.append(&browse);
    (labeled(label, &inner), entry, browse)
}

/// A dropdown pre-selected to `selected` (by matching text), if present.
pub fn dropdown(options: &[&str], selected: Option<&str>) -> DropDown {
    let dd = DropDown::from_strings(options);
    if let Some(text) = selected {
        if let Some(i) = options.iter().position(|o| *o == text) {
            dd.set_selected(i as u32);
        }
    }
    dd
}

/// Dropdown row, pre-selected to `selected` if its text is among `options`.
pub fn dropdown_row(label: &str, options: &[&str], selected: Option<&str>) -> (GtkBox, DropDown) {
    let dd = dropdown(options, selected);
    (labeled(label, &dd), dd)
}

/// `Some(trimmed)` unless the trimmed text is empty.
pub fn none_if_empty(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// `&str` view of an optional string, for pre-filling an entry.
pub fn opt(value: &Option<String>) -> &str {
    value.as_deref().unwrap_or("")
}

/// Whole-buffer text of a `TextView`.
pub fn textview_text(view: &TextView) -> String {
    let buffer = view.buffer();
    buffer
        .text(&buffer.start_iter(), &buffer.end_iter(), false)
        .to_string()
}

/// The currently selected option's text (`None` if nothing is selected).
///
/// Reads the selected `StringObject` directly, so callers don't need to keep
/// the original options slice around for read-back.
pub fn dropdown_selected(dd: &DropDown) -> Option<String> {
    dd.selected_item()
        .and_downcast::<gtk::StringObject>()
        .map(|s| s.string().to_string())
}
