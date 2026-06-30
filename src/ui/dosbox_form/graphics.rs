//! Graphics tab: output backend, machine, memory, scaler, aspect.

use gtk::prelude::*;
use gtk::{Box as GtkBox, DropDown, Entry};

use super::rows::{bool_opt, cfg_bool, cfg_opt, config_row, Ctx, DEFAULT};
use crate::config::dosbox_conf::DosboxConfig;
use crate::ui::widgets;

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
const SCALER_OPTS: [&str; 4] = [DEFAULT, "none", "normal2x", "normal3x"];
const ASPECT_OPTS: [&str; 3] = [DEFAULT, "on", "off"];
const MEMSIZE_PRESETS: [&str; 6] = ["1", "4", "8", "16", "32", "64"];

const DEF_OUTPUT: &str = "opengl";
const DEF_MACHINE: &str = "svga_s3";
const DEF_MEMSIZE: &str = "16";
const DEF_SCALER: &str = "none";
const DEF_ASPECT: &str = "auto";

/// Graphics-tab widgets, read back by [`Widgets::apply`].
pub(super) struct Widgets {
    output: DropDown,
    machine: DropDown,
    memsize: Entry,
    scaler: DropDown,
    aspect: DropDown,
}

/// Build the Graphics page and its read-back widgets.
pub(super) fn build(config: &DosboxConfig, ctx: &Ctx) -> (GtkBox, Widgets) {
    let page = widgets::page();

    let (row, output) = config_row(
        "Output",
        &OUTPUT_OPTS,
        config.output.as_deref(),
        &ctx.sentinel(DEF_OUTPUT),
    );
    page.append(&row);
    let (row, machine) = config_row(
        "Machine",
        &MACHINE_OPTS,
        config.machine.as_deref(),
        &ctx.sentinel(DEF_MACHINE),
    );
    page.append(&row);
    let memsize_cur = config.memsize.map(|v| v.to_string()).unwrap_or_default();
    let (row, memsize) = widgets::combo_row(
        "Memory (MB)",
        &MEMSIZE_PRESETS,
        &memsize_cur,
        &ctx.placeholder(DEF_MEMSIZE),
    );
    page.append(&row);
    let (row, scaler) = config_row(
        "Scaler",
        &SCALER_OPTS,
        config.scaler.as_deref(),
        &ctx.sentinel(DEF_SCALER),
    );
    page.append(&row);
    let (row, aspect) = config_row(
        "Aspect correction",
        &ASPECT_OPTS,
        bool_opt(config.aspect),
        &ctx.sentinel(DEF_ASPECT),
    );
    page.append(&row);

    (
        page,
        Widgets {
            output,
            machine,
            memsize,
            scaler,
            aspect,
        },
    )
}

impl Widgets {
    /// Write the Graphics fields into `cfg`.
    pub(super) fn apply(&self, cfg: &mut DosboxConfig) {
        cfg.output = cfg_opt(&self.output);
        cfg.machine = cfg_opt(&self.machine);
        cfg.memsize = widgets::none_if_empty(&self.memsize.text()).and_then(|s| s.parse().ok());
        cfg.scaler = cfg_opt(&self.scaler);
        cfg.aspect = cfg_bool(&self.aspect);
    }
}
