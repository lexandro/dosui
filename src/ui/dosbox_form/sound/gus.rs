//! Gravis UltraSound group: enable plus the IO triple.

use gtk::prelude::*;
use gtk::{Box as GtkBox, DropDown};

use super::super::rows::{bool_opt, cfg_bool, cfg_opt, config_row, heading, Ctx, DEFAULT};
use crate::config::dosbox_conf::DosboxConfig;

const GUS_OPTS: [&str; 3] = [DEFAULT, "on", "off"];
const GUSBASE_OPTS: [&str; 7] = [DEFAULT, "210", "220", "230", "240", "250", "260"];
const GUSIRQ_OPTS: [&str; 8] = [DEFAULT, "2", "3", "5", "7", "11", "12", "15"];
const GUSDMA_OPTS: [&str; 6] = [DEFAULT, "1", "3", "5", "6", "7"];

const DEF_GUS: &str = "off";
const DEF_GUSBASE: &str = "240";
const DEF_GUSIRQ: &str = "5";
const DEF_GUSDMA: &str = "3";

pub(super) struct Widgets {
    gus: DropDown,
    gusbase: DropDown,
    gusirq: DropDown,
    gusdma: DropDown,
}

pub(super) fn append(page: &GtkBox, config: &DosboxConfig, ctx: &Ctx) -> Widgets {
    page.append(&heading("Gravis UltraSound"));
    let (row, gus) = config_row(
        "Enable",
        &GUS_OPTS,
        bool_opt(config.gus),
        &ctx.sentinel(DEF_GUS),
    );
    page.append(&row);
    let (row, gusbase) = config_row(
        "Port",
        &GUSBASE_OPTS,
        config.gusbase.as_deref(),
        &ctx.sentinel(DEF_GUSBASE),
    );
    page.append(&row);
    let (row, gusirq) = config_row(
        "IRQ",
        &GUSIRQ_OPTS,
        config.gusirq.as_deref(),
        &ctx.sentinel(DEF_GUSIRQ),
    );
    page.append(&row);
    let (row, gusdma) = config_row(
        "DMA",
        &GUSDMA_OPTS,
        config.gusdma.as_deref(),
        &ctx.sentinel(DEF_GUSDMA),
    );
    page.append(&row);

    Widgets {
        gus,
        gusbase,
        gusirq,
        gusdma,
    }
}

impl Widgets {
    pub(super) fn apply(&self, cfg: &mut DosboxConfig) {
        cfg.gus = cfg_bool(&self.gus);
        cfg.gusbase = cfg_opt(&self.gusbase);
        cfg.gusirq = cfg_opt(&self.gusirq);
        cfg.gusdma = cfg_opt(&self.gusdma);
    }
}
