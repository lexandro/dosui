//! PC speaker / Tandy group ([speaker] section).

use gtk::prelude::*;
use gtk::{Box as GtkBox, DropDown};

use super::super::rows::{cfg_opt, config_row, heading, Ctx, DEFAULT};
use crate::config::dosbox_conf::DosboxConfig;

const TANDY_OPTS: [&str; 3] = [DEFAULT, "auto", "on"];
const PCSPEAKER_OPTS: [&str; 4] = [DEFAULT, "impulse", "discrete", "none"];

const DEF_TANDY: &str = "auto";
const DEF_PCSPEAKER: &str = "impulse";

pub(super) struct Widgets {
    tandy: DropDown,
    pcspeaker: DropDown,
}

pub(super) fn append(page: &GtkBox, config: &DosboxConfig, ctx: &Ctx) -> Widgets {
    page.append(&heading("PC speaker / Tandy"));
    let (row, pcspeaker) = config_row(
        "PC speaker",
        &PCSPEAKER_OPTS,
        config.pcspeaker.as_deref(),
        &ctx.sentinel(DEF_PCSPEAKER),
    );
    page.append(&row);
    let (row, tandy) = config_row(
        "Tandy/PCjr",
        &TANDY_OPTS,
        config.tandy.as_deref(),
        &ctx.sentinel(DEF_TANDY),
    );
    page.append(&row);

    Widgets { tandy, pcspeaker }
}

impl Widgets {
    pub(super) fn apply(&self, cfg: &mut DosboxConfig) {
        cfg.tandy = cfg_opt(&self.tandy);
        cfg.pcspeaker = cfg_opt(&self.pcspeaker);
    }
}
