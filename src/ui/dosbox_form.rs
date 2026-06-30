//! Reusable DOSBox-settings form: the CPU / Graphics / Sound / Advanced tabs.
//!
//! Builds notebook pages from a [`DosboxConfig`] and reads them back with
//! [`DosboxForm::collect`]. Shared by the profile editor (per-profile overrides)
//! and the settings dialog (global defaults). Optional dropdowns use a
//! `(default)`/`(inherit)` sentinel meaning "leave this key unset".
//!
//! Over the 150-line soft cap by design: one cohesive widget component (build +
//! read-back of the same struct) plus its inline tests.

use gtk::prelude::*;
use gtk::{Box as GtkBox, DropDown, Entry, Label, ScrolledWindow, TextView};
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
const CORE_OPTS: [&str; 5] = [DEFAULT, "auto", "normal", "dynamic", "simple"];
const CPUTYPE_OPTS: [&str; 6] = [
    DEFAULT,
    "auto",
    "386",
    "386_slow",
    "486_slow",
    "pentium_slow",
];
const SCALER_OPTS: [&str; 4] = [DEFAULT, "none", "normal2x", "normal3x"];
const ASPECT_OPTS: [&str; 3] = [DEFAULT, "on", "off"];
const SBTYPE_OPTS: [&str; 9] = [
    DEFAULT, "gb", "sb1", "sb2", "sbpro1", "sbpro2", "sb16", "ess", "none",
];
const SBBASE_OPTS: [&str; 9] = [
    DEFAULT, "220", "240", "260", "280", "2a0", "2c0", "2e0", "300",
];
const SBIRQ_OPTS: [&str; 8] = [DEFAULT, "3", "5", "7", "9", "10", "11", "12"];
const SBDMA_OPTS: [&str; 7] = [DEFAULT, "0", "1", "3", "5", "6", "7"];
const GUS_OPTS: [&str; 3] = [DEFAULT, "on", "off"];
const GUSBASE_OPTS: [&str; 7] = [DEFAULT, "210", "220", "230", "240", "250", "260"];
const GUSIRQ_OPTS: [&str; 8] = [DEFAULT, "2", "3", "5", "7", "11", "12", "15"];
const GUSDMA_OPTS: [&str; 6] = [DEFAULT, "1", "3", "5", "6", "7"];
const MIDI_OPTS: [&str; 5] = [DEFAULT, "auto", "mt32", "fluidsynth", "none"];

// Presets for the editable (free-text) value fields — the user can also type
// any value (e.g. an arbitrary cycles count).
const CYCLES_PRESETS: [&str; 6] = [
    "auto",
    "max",
    "fixed 3000",
    "fixed 10000",
    "fixed 20000",
    "30000",
];
const MEMSIZE_PRESETS: [&str; 6] = ["1", "4", "8", "16", "32", "64"];
const RATE_PRESETS: [&str; 5] = ["22050", "32000", "44100", "48000", "49716"];

// DOSBox-staging built-in defaults, surfaced next to "Default" so it's clear
// what leaving a field unset actually does.
const DEF_CORE: &str = "auto";
const DEF_CPUTYPE: &str = "auto";
const DEF_CYCLES: &str = "auto";
const DEF_OUTPUT: &str = "opengl";
const DEF_MACHINE: &str = "svga_s3";
const DEF_MEMSIZE: &str = "16";
const DEF_ASPECT: &str = "auto";
const DEF_SCALER: &str = "none";
const DEF_SBTYPE: &str = "sb16";
const DEF_SBBASE: &str = "220";
const DEF_SBIRQ: &str = "7";
const DEF_SBDMA: &str = "1";
const DEF_SBHDMA: &str = "5";
const DEF_RATE: &str = "48000";
const DEF_GUS: &str = "off";
const DEF_GUSBASE: &str = "240";
const DEF_GUSIRQ: &str = "5";
const DEF_GUSDMA: &str = "3";
const DEF_MIDI: &str = "auto";

/// The DOSBox tab pages plus the widgets read on save.
pub struct DosboxForm {
    pub cpu_page: GtkBox,
    pub graphics_page: GtkBox,
    pub sound_page: GtkBox,
    pub advanced_page: GtkBox,

    output: DropDown,
    machine: DropDown,
    memsize: Entry,
    scaler: DropDown,
    aspect: DropDown,
    core: DropDown,
    cputype: DropDown,
    cycles: Entry,
    sbtype: DropDown,
    sbbase: DropDown,
    sbirq: DropDown,
    sbdma: DropDown,
    sbhdma: DropDown,
    rate: Entry,
    gus: DropDown,
    gusbase: DropDown,
    gusirq: DropDown,
    gusdma: DropDown,
    mididevice: DropDown,
    passthrough: TextView,
    preview: TextView,
}

impl DosboxForm {
    /// Build the tabs from `config`. `default_label` is the sentinel text shown
    /// for unset values ("(default)" for global defaults, "(inherit)" for a profile).
    pub fn new(config: &DosboxConfig, default_label: &str, show_builtin: bool) -> DosboxForm {
        // Unset-dropdown text: append the DOSBox built-in value (Settings only)
        // so "Default" actually says what it does.
        let sentinel = |builtin: &str| -> String {
            if show_builtin {
                format!("{default_label} · {builtin}")
            } else {
                default_label.to_string()
            }
        };
        // Placeholder shown in an empty editable field.
        let placeholder = |builtin: &str| -> String {
            if show_builtin {
                builtin.to_string()
            } else {
                default_label.to_string()
            }
        };

        let cpu_page = widgets::page();
        let (row, core) = config_row(
            "Core",
            &CORE_OPTS,
            config.core.as_deref(),
            &sentinel(DEF_CORE),
        );
        cpu_page.append(&row);
        let (row, cputype) = config_row(
            "CPU type",
            &CPUTYPE_OPTS,
            config.cputype.as_deref(),
            &sentinel(DEF_CPUTYPE),
        );
        cpu_page.append(&row);
        let (row, cycles) = widgets::combo_row(
            "Cycles",
            &CYCLES_PRESETS,
            widgets::opt(&config.cycles),
            &placeholder(DEF_CYCLES),
        );
        cpu_page.append(&row);

        let graphics_page = widgets::page();
        let (row, output) = config_row(
            "Output",
            &OUTPUT_OPTS,
            config.output.as_deref(),
            &sentinel(DEF_OUTPUT),
        );
        graphics_page.append(&row);
        let (row, machine) = config_row(
            "Machine",
            &MACHINE_OPTS,
            config.machine.as_deref(),
            &sentinel(DEF_MACHINE),
        );
        graphics_page.append(&row);
        let memsize_cur = config.memsize.map(|v| v.to_string()).unwrap_or_default();
        let (row, memsize) = widgets::combo_row(
            "Memory (MB)",
            &MEMSIZE_PRESETS,
            &memsize_cur,
            &placeholder(DEF_MEMSIZE),
        );
        graphics_page.append(&row);
        let (row, scaler) = config_row(
            "Scaler",
            &SCALER_OPTS,
            config.scaler.as_deref(),
            &sentinel(DEF_SCALER),
        );
        graphics_page.append(&row);
        let aspect_cur = config.aspect.map(|b| if b { "on" } else { "off" });
        let (row, aspect) = config_row(
            "Aspect correction",
            &ASPECT_OPTS,
            aspect_cur,
            &sentinel(DEF_ASPECT),
        );
        graphics_page.append(&row);

        let sound_page = widgets::page();

        sound_page.append(&heading("Sound Blaster"));
        let (row, sbtype) = config_row(
            "Type",
            &SBTYPE_OPTS,
            config.sbtype.as_deref(),
            &sentinel(DEF_SBTYPE),
        );
        sound_page.append(&row);
        let (row, sbbase) = config_row(
            "Port",
            &SBBASE_OPTS,
            config.sbbase.as_deref(),
            &sentinel(DEF_SBBASE),
        );
        sound_page.append(&row);
        let (row, sbirq) = config_row(
            "IRQ",
            &SBIRQ_OPTS,
            config.sbirq.as_deref(),
            &sentinel(DEF_SBIRQ),
        );
        sound_page.append(&row);
        let (row, sbdma) = config_row(
            "DMA",
            &SBDMA_OPTS,
            config.sbdma.as_deref(),
            &sentinel(DEF_SBDMA),
        );
        sound_page.append(&row);
        let (row, sbhdma) = config_row(
            "High DMA",
            &SBDMA_OPTS,
            config.sbhdma.as_deref(),
            &sentinel(DEF_SBHDMA),
        );
        sound_page.append(&row);

        sound_page.append(&heading("Gravis UltraSound"));
        let gus_cur = config.gus.map(|b| if b { "on" } else { "off" });
        let (row, gus) = config_row("Enable", &GUS_OPTS, gus_cur, &sentinel(DEF_GUS));
        sound_page.append(&row);
        let (row, gusbase) = config_row(
            "Port",
            &GUSBASE_OPTS,
            config.gusbase.as_deref(),
            &sentinel(DEF_GUSBASE),
        );
        sound_page.append(&row);
        let (row, gusirq) = config_row(
            "IRQ",
            &GUSIRQ_OPTS,
            config.gusirq.as_deref(),
            &sentinel(DEF_GUSIRQ),
        );
        sound_page.append(&row);
        let (row, gusdma) = config_row(
            "DMA",
            &GUSDMA_OPTS,
            config.gusdma.as_deref(),
            &sentinel(DEF_GUSDMA),
        );
        sound_page.append(&row);

        sound_page.append(&heading("Mixer"));
        let rate_cur = config.rate.map(|v| v.to_string()).unwrap_or_default();
        let (row, rate) = widgets::combo_row(
            "Rate (Hz)",
            &RATE_PRESETS,
            &rate_cur,
            &placeholder(DEF_RATE),
        );
        sound_page.append(&row);

        sound_page.append(&heading("MIDI"));
        let (row, mididevice) = config_row(
            "Device",
            &MIDI_OPTS,
            config.mididevice.as_deref(),
            &sentinel(DEF_MIDI),
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
            sbbase,
            sbirq,
            sbdma,
            sbhdma,
            rate,
            gus,
            gusbase,
            gusirq,
            gusdma,
            mididevice,
            passthrough,
            preview,
        }
    }

    /// Read the tabs into a [`DosboxConfig`].
    pub fn collect(&self) -> DosboxConfig {
        DosboxConfig {
            output: cfg_opt(&self.output),
            machine: cfg_opt(&self.machine),
            memsize: widgets::none_if_empty(&self.memsize.text()).and_then(|s| s.parse().ok()),
            core: cfg_opt(&self.core),
            cputype: cfg_opt(&self.cputype),
            cycles: widgets::none_if_empty(&self.cycles.text()),
            aspect: cfg_bool(&self.aspect),
            scaler: cfg_opt(&self.scaler),
            sbtype: cfg_opt(&self.sbtype),
            sbbase: cfg_opt(&self.sbbase),
            sbirq: cfg_opt(&self.sbirq),
            sbdma: cfg_opt(&self.sbdma),
            sbhdma: cfg_opt(&self.sbhdma),
            rate: widgets::none_if_empty(&self.rate.text()).and_then(|s| s.parse().ok()),
            gus: cfg_bool(&self.gus),
            gusbase: cfg_opt(&self.gusbase),
            gusirq: cfg_opt(&self.gusirq),
            gusdma: cfg_opt(&self.gusdma),
            mididevice: cfg_opt(&self.mididevice),
            passthrough: parse_passthrough(&widgets::textview_text(&self.passthrough)),
        }
    }

    /// Set the read-only preview text (the caller decides what to render).
    pub fn set_preview(&self, text: &str) {
        self.preview.buffer().set_text(text);
    }
}

/// A config dropdown row: curated `base` options (sentinel at index 0 relabelled
/// to `sentinel`), plus the current value if it isn't already listed.
fn config_row(
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

/// A bold section heading inside a tab (e.g. "Sound Blaster").
fn heading(text: &str) -> Label {
    Label::builder()
        .label(text)
        .halign(gtk::Align::Start)
        .margin_top(6)
        .css_classes(["heading"])
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
