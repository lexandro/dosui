//! CPU tab: core, CPU type, and cycles.

use gtk::prelude::*;
use gtk::{Box as GtkBox, DropDown, Entry};

use super::rows::{cfg_opt, config_row, Ctx, DEFAULT};
use crate::config::dosbox_conf::DosboxConfig;
use crate::ui::widgets;

const CORE_OPTS: [&str; 5] = [DEFAULT, "auto", "normal", "dynamic", "simple"];
const CPUTYPE_OPTS: [&str; 6] = [
    DEFAULT,
    "auto",
    "386",
    "386_slow",
    "486_slow",
    "pentium_slow",
];
const CYCLES_PRESETS: [&str; 6] = [
    "auto",
    "max",
    "fixed 3000",
    "fixed 10000",
    "fixed 20000",
    "30000",
];

const DEF_CORE: &str = "auto";
const DEF_CPUTYPE: &str = "auto";
const DEF_CYCLES: &str = "auto";

/// CPU-tab widgets, read back by [`Widgets::apply`].
pub(super) struct Widgets {
    core: DropDown,
    cputype: DropDown,
    cycles: Entry,
}

/// Build the CPU page and its read-back widgets.
pub(super) fn build(config: &DosboxConfig, ctx: &Ctx) -> (GtkBox, Widgets) {
    let page = widgets::page();

    let (row, core) = config_row(
        "Core",
        &CORE_OPTS,
        config.core.as_deref(),
        &ctx.sentinel(DEF_CORE),
    );
    page.append(&row);
    let (row, cputype) = config_row(
        "CPU type",
        &CPUTYPE_OPTS,
        config.cputype.as_deref(),
        &ctx.sentinel(DEF_CPUTYPE),
    );
    page.append(&row);
    let (row, cycles) = widgets::combo_row(
        "Cycles",
        &CYCLES_PRESETS,
        widgets::opt(&config.cycles),
        &ctx.placeholder(DEF_CYCLES),
    );
    page.append(&row);

    (
        page,
        Widgets {
            core,
            cputype,
            cycles,
        },
    )
}

impl Widgets {
    /// Write the CPU fields into `cfg`.
    pub(super) fn apply(&self, cfg: &mut DosboxConfig) {
        cfg.core = cfg_opt(&self.core);
        cfg.cputype = cfg_opt(&self.cputype);
        cfg.cycles = widgets::none_if_empty(&self.cycles.text());
    }
}
