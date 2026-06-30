//! Graphics tab: output backend, fullscreen/vsync, machine, memory, shader, aspect.

use gtk::prelude::*;
use gtk::{Box as GtkBox, DropDown, Entry};

use super::rows::{bool_opt, cfg_bool, cfg_opt, config_row, Ctx, DEFAULT};
use crate::config::dosbox_conf::DosboxConfig;
use crate::ui::widgets;

const OUTPUT_OPTS: [&str; 4] = [DEFAULT, "texture", "texturenb", "opengl"];
const ONOFF_OPTS: [&str; 3] = [DEFAULT, "on", "off"];
const VSYNC_OPTS: [&str; 5] = [DEFAULT, "auto", "on", "adaptive", "off"];
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
const ASPECT_OPTS: [&str; 3] = [DEFAULT, "on", "off"];
// glshader is open-ended (dozens of named shaders); offer the common ones and
// let the user type any name. Exotic shaders still work via the Advanced tab.
const GLSHADER_PRESETS: [&str; 5] = [
    "none",
    "sharp",
    "crt-auto",
    "crt-auto-machine",
    "crt-auto-arcade",
];

const DEF_OUTPUT: &str = "opengl";
const DEF_FULLSCREEN: &str = "off";
const DEF_VSYNC: &str = "auto";
const DEF_MACHINE: &str = "svga_s3";
const DEF_GLSHADER: &str = "crt-auto";
const DEF_ASPECT: &str = "auto";

/// Graphics-tab widgets, read back by [`Widgets::apply`].
pub(super) struct Widgets {
    output: DropDown,
    fullscreen: DropDown,
    vsync: DropDown,
    machine: DropDown,
    glshader: Entry,
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
    let (row, fullscreen) = config_row(
        "Fullscreen",
        &ONOFF_OPTS,
        bool_opt(config.fullscreen),
        &ctx.sentinel(DEF_FULLSCREEN),
    );
    page.append(&row);
    let (row, vsync) = config_row(
        "VSync",
        &VSYNC_OPTS,
        config.vsync.as_deref(),
        &ctx.sentinel(DEF_VSYNC),
    );
    page.append(&row);
    let (row, machine) = config_row(
        "Machine",
        &MACHINE_OPTS,
        config.machine.as_deref(),
        &ctx.sentinel(DEF_MACHINE),
    );
    page.append(&row);
    let (row, glshader) = widgets::combo_row(
        "Shader",
        &GLSHADER_PRESETS,
        widgets::opt(&config.glshader),
        &ctx.placeholder(DEF_GLSHADER),
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
            fullscreen,
            vsync,
            machine,
            glshader,
            aspect,
        },
    )
}

impl Widgets {
    /// Write the Graphics fields into `cfg`.
    pub(super) fn apply(&self, cfg: &mut DosboxConfig) {
        cfg.output = cfg_opt(&self.output);
        cfg.fullscreen = cfg_bool(&self.fullscreen);
        cfg.vsync = cfg_opt(&self.vsync);
        cfg.machine = cfg_opt(&self.machine);
        cfg.glshader = widgets::none_if_empty(&self.glshader.text());
        cfg.aspect = cfg_bool(&self.aspect);
    }
}
