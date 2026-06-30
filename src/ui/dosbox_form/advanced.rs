//! Advanced tab: free-form `section.key = value` passthrough plus a read-only
//! preview of the generated `dosbox.conf`.

use gtk::prelude::*;
use gtk::{Box as GtkBox, Label, ScrolledWindow, TextView};
use indexmap::IndexMap;

use crate::config::dosbox_conf::DosboxConfig;
use crate::ui::widgets;

/// Advanced-tab widgets, read back by [`Widgets::apply`].
pub(super) struct Widgets {
    passthrough: TextView,
    pub(super) preview: TextView,
}

/// Build the Advanced page and its read-back widgets.
pub(super) fn build(config: &DosboxConfig) -> (GtkBox, Widgets) {
    let page = widgets::page();
    page.append(&hint(
        "Advanced keys — one per line, e.g. cpu.cycleup = 500",
    ));

    let passthrough = TextView::new();
    passthrough.set_monospace(true);
    passthrough
        .buffer()
        .set_text(&serialize_passthrough(&config.passthrough));
    page.append(&scroller(&passthrough, 120));

    page.append(&hint("Generated dosbox.conf preview"));
    let preview = TextView::builder().editable(false).monospace(true).build();
    page.append(&scroller(&preview, 150));

    (
        page,
        Widgets {
            passthrough,
            preview,
        },
    )
}

impl Widgets {
    /// Write the passthrough map into `cfg`.
    pub(super) fn apply(&self, cfg: &mut DosboxConfig) {
        cfg.passthrough = parse_passthrough(&widgets::textview_text(&self.passthrough));
    }
}

/// Parse "section.key = value" lines into the passthrough map; blanks/`#`/bad lines skipped.
fn parse_passthrough(text: &str) -> IndexMap<String, IndexMap<String, String>> {
    let mut map: IndexMap<String, IndexMap<String, String>> = IndexMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((lhs, value)) = line.split_once('=') else {
            continue;
        };
        let Some((section, key)) = lhs.trim().split_once('.') else {
            continue;
        };
        map.entry(section.trim().to_string())
            .or_default()
            .insert(key.trim().to_string(), value.trim().to_string());
    }
    map
}

/// Inverse of [`parse_passthrough`].
fn serialize_passthrough(map: &IndexMap<String, IndexMap<String, String>>) -> String {
    let mut out = String::new();
    for (section, keys) in map {
        for (key, value) in keys {
            out.push_str(&format!("{section}.{key} = {value}\n"));
        }
    }
    out
}

fn hint(text: &str) -> Label {
    Label::builder()
        .label(text)
        .halign(gtk::Align::Start)
        .build()
}

fn scroller(child: &TextView, min_height: i32) -> ScrolledWindow {
    ScrolledWindow::builder()
        .child(child)
        .min_content_height(min_height)
        .vexpand(true)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passthrough_round_trips() {
        let text = "cpu.cycleup = 500\nrender.glshader = crt\n";
        let map = parse_passthrough(text);
        assert_eq!(map["cpu"]["cycleup"], "500");
        assert_eq!(map["render"]["glshader"], "crt");
        assert_eq!(serialize_passthrough(&map), text);
    }

    #[test]
    fn passthrough_skips_blank_and_malformed_lines() {
        let map = parse_passthrough("\n# comment\nnonsense\ncpu.core = dynamic\n");
        assert_eq!(map.len(), 1);
        assert_eq!(map["cpu"]["core"], "dynamic");
    }
}
