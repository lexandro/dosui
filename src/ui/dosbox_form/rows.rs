//! Shared building blocks for the DOSBox-settings tabs.
//!
//! [`Ctx`] carries how unset values are presented (the `(default)`/`(inherit)`
//! sentinel, optionally suffixed with the DOSBox built-in value). The `*_row`
//! and `cfg_*` helpers turn curated option lists into dropdown rows and read
//! them back, treating index 0 (the sentinel) as "leave this key unset".

use gtk::{Box as GtkBox, DropDown, Label};

use crate::ui::widgets;

/// Sentinel first option in every list: "don't set this key".
pub(super) const DEFAULT: &str = "(default)";

/// How unset values are labelled, shared by all tabs.
pub(super) struct Ctx {
    /// Text for the "unset" choice: `(default)` (global) or `(inherit)` (profile).
    pub label: String,
    /// Append the DOSBox built-in value to the sentinel / use it as placeholder.
    pub show_builtin: bool,
}

impl Ctx {
    /// Unset-dropdown text: append the built-in (Settings only) so the sentinel
    /// actually says what leaving it unset does, e.g. `(default) · svga_s3`.
    pub(super) fn sentinel(&self, builtin: &str) -> String {
        if self.show_builtin {
            format!("{} · {builtin}", self.label)
        } else {
            self.label.clone()
        }
    }

    /// Placeholder shown in an empty editable field.
    pub(super) fn placeholder(&self, builtin: &str) -> String {
        if self.show_builtin {
            builtin.to_string()
        } else {
            self.label.clone()
        }
    }
}

/// A config dropdown row: curated `base` options (sentinel at index 0 relabelled
/// to `sentinel`), plus the current value if it isn't already listed.
pub(super) fn config_row(
    label: &str,
    base: &[&str],
    current: Option<&str>,
    sentinel: &str,
) -> (GtkBox, DropDown) {
    let current = current.filter(|c| !c.is_empty());
    let mut opts: Vec<&str> = base.to_vec();
    opts[0] = sentinel; // relabel the sentinel (index 0 stays the "unset" option)
    if let Some(c) = current {
        if !opts.contains(&c) {
            opts.insert(1, c);
        }
    }
    widgets::dropdown_row(label, &opts, Some(current.unwrap_or(sentinel)))
}

/// Selected dropdown text, treating the sentinel (always index 0) as `None`.
pub(super) fn cfg_opt(dd: &DropDown) -> Option<String> {
    if dd.selected() == 0 {
        None
    } else {
        widgets::dropdown_selected(dd)
    }
}

/// `on`/`off` dropdown -> `Option<bool>` (sentinel -> `None`).
pub(super) fn cfg_bool(dd: &DropDown) -> Option<bool> {
    match widgets::dropdown_selected(dd).as_deref() {
        Some("on") => Some(true),
        Some("off") => Some(false),
        _ => None,
    }
}

/// `on`/`off` for pre-selecting a bool dropdown (`None` -> unset/sentinel).
pub(super) fn bool_opt(value: Option<bool>) -> Option<&'static str> {
    value.map(|b| if b { "on" } else { "off" })
}

/// A bold section heading inside a tab (e.g. "Sound Blaster").
pub(super) fn heading(text: &str) -> Label {
    Label::builder()
        .label(text)
        .halign(gtk::Align::Start)
        .margin_top(6)
        .css_classes(["heading"])
        .build()
}
