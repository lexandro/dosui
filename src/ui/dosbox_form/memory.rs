//! Memory tab: RAM size, video memory, and the XMS/EMS/UMB managers.

use gtk::prelude::*;
use gtk::{Box as GtkBox, DropDown, Entry};

use super::rows::{bool_opt, cfg_bool, cfg_opt, config_row, heading, Ctx, DEFAULT};
use crate::config::dosbox_conf::DosboxConfig;
use crate::ui::widgets;

const MEMSIZE_PRESETS: [&str; 6] = ["1", "4", "8", "16", "32", "64"];
const VMEMSIZE_PRESETS: [&str; 5] = ["auto", "1", "2", "4", "8"];
const ONOFF_OPTS: [&str; 3] = [DEFAULT, "on", "off"];
const EMS_OPTS: [&str; 5] = [DEFAULT, "true", "false", "emsboard", "emm386"];

const DEF_MEMSIZE: &str = "16";
const DEF_VMEMSIZE: &str = "auto";
const DEF_XMS: &str = "on";
const DEF_EMS: &str = "true";
const DEF_UMB: &str = "on";

/// Memory-tab widgets, read back by [`Widgets::apply`].
pub(super) struct Widgets {
    memsize: Entry,
    vmemsize: Entry,
    xms: DropDown,
    ems: DropDown,
    umb: DropDown,
}

/// Build the Memory page and its read-back widgets.
pub(super) fn build(config: &DosboxConfig, ctx: &Ctx) -> (GtkBox, Widgets) {
    let page = widgets::page();

    let memsize_cur = config.memsize.map(|v| v.to_string()).unwrap_or_default();
    let (row, memsize) = widgets::combo_row(
        "Memory (MB)",
        &MEMSIZE_PRESETS,
        &memsize_cur,
        &ctx.placeholder(DEF_MEMSIZE),
    );
    page.append(&row);
    let (row, vmemsize) = widgets::combo_row(
        "Video memory (MB)",
        &VMEMSIZE_PRESETS,
        widgets::opt(&config.vmemsize),
        &ctx.placeholder(DEF_VMEMSIZE),
    );
    page.append(&row);

    page.append(&heading("DOS memory managers"));
    let (row, xms) = config_row(
        "XMS",
        &ONOFF_OPTS,
        bool_opt(config.xms),
        &ctx.sentinel(DEF_XMS),
    );
    page.append(&row);
    let (row, ems) = config_row(
        "EMS",
        &EMS_OPTS,
        config.ems.as_deref(),
        &ctx.sentinel(DEF_EMS),
    );
    page.append(&row);
    let (row, umb) = config_row(
        "UMB",
        &ONOFF_OPTS,
        bool_opt(config.umb),
        &ctx.sentinel(DEF_UMB),
    );
    page.append(&row);

    (
        page,
        Widgets {
            memsize,
            vmemsize,
            xms,
            ems,
            umb,
        },
    )
}

impl Widgets {
    /// Write the Memory fields into `cfg`.
    pub(super) fn apply(&self, cfg: &mut DosboxConfig) {
        cfg.memsize = widgets::none_if_empty(&self.memsize.text()).and_then(|s| s.parse().ok());
        cfg.vmemsize = widgets::none_if_empty(&self.vmemsize.text());
        cfg.xms = cfg_bool(&self.xms);
        cfg.ems = cfg_opt(&self.ems);
        cfg.umb = cfg_bool(&self.umb);
    }
}
