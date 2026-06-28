//! Reusable DOSBox-settings form: the CPU / Graphics / Sound / Advanced tabs.
//!
//! Builds notebook pages from a [`DosboxConfig`] and reads them back with
//! [`DosboxForm::collect`]. Shared by the profile editor (per-profile overrides)
//! and the settings dialog (global defaults). Optional dropdowns use a
//! `(default)`/`(inherit)` sentinel meaning "leave this key unset".

use gtk::prelude::*;
use gtk::{Box as GtkBox, DropDown, Label, ScrolledWindow, TextView};
use indexmap::IndexMap;

use crate::config::dosbox_conf::DosboxConfig;
use crate::ui::widgets;

/// Sentinel first option: "don't set this key" (caller chooses the visible text).
const DEFAULT: &str = "(default)";

// Curated option lists. The config's current value is added if missing, so
// arbitrary values survive a round-trip.
const OUTPUT_OPTS: [&str; 4] = [DEFAULT, "texture", "texturenb", "opengl"];
const MACHINE_OPTS: [&str; 8] = [
    DEFAULT,
    "svga_s3",
    "svga_et4000",
    "vesa_nolfb",
    "vgaonly",
    "ega",
    "cga",
    "hercules",
];
const MEMSIZE_OPTS: [&str; 7] = [DEFAULT, "1", "4", "8", "16", "32", "64"];
const CORE_OPTS: [&str; 5] = [DEFAULT, "auto", "normal", "dynamic", "simple"];
const CPUTYPE_OPTS: [&str; 6] = [
    DEFAULT,
    "auto",
    "386",
    "386_slow",
    "486_slow",
    "pentium_slow",
];
const CYCLES_OPTS: [&str; 7] = [
    DEFAULT,
    "auto",
    "max",
    "fixed 3000",
    "fixed 10000",
    "fixed 20000",
    "fixed 30000",
];
const SCALER_OPTS: [&str; 4] = [DEFAULT, "none", "normal2x", "normal3x"];
const ASPECT_OPTS: [&str; 3] = [DEFAULT, "on", "off"];
const SBTYPE_OPTS: [&str; 6] = [DEFAULT, "sb16", "sbpro2", "sb2", "gb", "none"];
const RATE_OPTS: [&str; 6] = [DEFAULT, "22050", "32000", "44100", "48000", "49716"];

/// The DOSBox tab pages plus the widgets read on save.
pub struct DosboxForm {
    pub cpu_page: GtkBox,
    pub graphics_page: GtkBox,
    pub sound_page: GtkBox,
    pub advanced_page: GtkBox,

    output: DropDown,
    machine: DropDown,
    memsize: DropDown,
    scaler: DropDown,
    aspect: DropDown,
    core: DropDown,
    cputype: DropDown,
    cycles: DropDown,
    sbtype: DropDown,
    rate: DropDown,
    passthrough: TextView,
    preview: TextView,
}

impl DosboxForm {
    /// Build the tabs from `config`. `default_label` is the sentinel text shown
    /// for unset values ("(default)" for global defaults, "(inherit)" for a profile).
    pub fn new(config: &DosboxConfig, default_label: &str) -> DosboxForm {
        let cpu_page = widgets::page();
        let (row, core) = config_row("Core", &CORE_OPTS, config.core.as_deref(), default_label);
        cpu_page.append(&row);
        let (row, cputype) = config_row(
            "CPU type",
            &CPUTYPE_OPTS,
            config.cputype.as_deref(),
            default_label,
        );
        cpu_page.append(&row);
        let (row, cycles) = config_row(
            "Cycles",
            &CYCLES_OPTS,
            config.cycles.as_deref(),
            default_label,
        );
        cpu_page.append(&row);

        let graphics_page = widgets::page();
        let (row, output) = config_row(
            "Output",
            &OUTPUT_OPTS,
            config.output.as_deref(),
            default_label,
        );
        graphics_page.append(&row);
        let (row, machine) = config_row(
            "Machine",
            &MACHINE_OPTS,
            config.machine.as_deref(),
            default_label,
        );
        graphics_page.append(&row);
        let memsize_cur = config.memsize.map(|v| v.to_string());
        let (row, memsize) = config_row(
            "Memory (MB)",
            &MEMSIZE_OPTS,
            memsize_cur.as_deref(),
            default_label,
        );
        graphics_page.append(&row);
        let (row, scaler) = config_row(
            "Scaler",
            &SCALER_OPTS,
            config.scaler.as_deref(),
            default_label,
        );
        graphics_page.append(&row);
        let aspect_cur = config.aspect.map(|b| if b { "on" } else { "off" });
        let (row, aspect) =
            config_row("Aspect correction", &ASPECT_OPTS, aspect_cur, default_label);
        graphics_page.append(&row);

        let sound_page = widgets::page();
        let (row, sbtype) = config_row(
            "Sound Blaster",
            &SBTYPE_OPTS,
            config.sbtype.as_deref(),
            default_label,
        );
        sound_page.append(&row);
        let rate_cur = config.rate.map(|v| v.to_string());
        let (row, rate) = config_row(
            "Mixer rate (Hz)",
            &RATE_OPTS,
            rate_cur.as_deref(),
            default_label,
        );
        sound_page.append(&row);

        let advanced_page = widgets::page();
        advanced_page.append(&hint(
            "Advanced keys — one per line, e.g. cpu.cycleup = 500",
        ));
        let passthrough = TextView::new();
        passthrough.set_monospace(true);
        passthrough
            .buffer()
            .set_text(&serialize_passthrough(&config.passthrough));
        advanced_page.append(&scroller(&passthrough, 120));
        advanced_page.append(&hint("Generated dosbox.conf preview"));
        let preview = TextView::builder().editable(false).monospace(true).build();
        advanced_page.append(&scroller(&preview, 150));

        DosboxForm {
            cpu_page,
            graphics_page,
            sound_page,
            advanced_page,
            output,
            machine,
            memsize,
            scaler,
            aspect,
            core,
            cputype,
            cycles,
            sbtype,
            rate,
            passthrough,
            preview,
        }
    }

    /// Read the tabs into a [`DosboxConfig`].
    pub fn collect(&self) -> DosboxConfig {
        DosboxConfig {
            output: cfg_opt(&self.output),
            machine: cfg_opt(&self.machine),
            memsize: cfg_opt(&self.memsize).and_then(|s| s.parse().ok()),
            core: cfg_opt(&self.core),
            cputype: cfg_opt(&self.cputype),
            cycles: cfg_opt(&self.cycles),
            aspect: cfg_bool(&self.aspect),
            scaler: cfg_opt(&self.scaler),
            sbtype: cfg_opt(&self.sbtype),
            rate: cfg_opt(&self.rate).and_then(|s| s.parse().ok()),
            passthrough: parse_passthrough(&widgets::textview_text(&self.passthrough)),
        }
    }

    /// Set the read-only preview text (the caller decides what to render).
    pub fn set_preview(&self, text: &str) {
        self.preview.buffer().set_text(text);
    }
}

/// A config dropdown row: curated `base` options (sentinel relabelled to
/// `default_label`), plus the current value if not listed.
fn config_row(
    label: &str,
    base: &[&str],
    current: Option<&str>,
    default_label: &str,
) -> (GtkBox, DropDown) {
    let current = current.filter(|c| !c.is_empty());
    let mut opts: Vec<&str> = base.to_vec();
    opts[0] = default_label; // relabel the sentinel
    if let Some(c) = current {
        if !opts.contains(&c) {
            opts.insert(1, c);
        }
    }
    widgets::dropdown_row(label, &opts, Some(current.unwrap_or(default_label)))
}

/// Selected dropdown text, treating the sentinel (always index 0) as `None`.
fn cfg_opt(dd: &DropDown) -> Option<String> {
    if dd.selected() == 0 {
        None
    } else {
        widgets::dropdown_selected(dd)
    }
}

/// `on`/`off` dropdown -> `Option<bool>` (sentinel -> `None`).
fn cfg_bool(dd: &DropDown) -> Option<bool> {
    match widgets::dropdown_selected(dd).as_deref() {
        Some("on") => Some(true),
        Some("off") => Some(false),
        _ => None,
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
