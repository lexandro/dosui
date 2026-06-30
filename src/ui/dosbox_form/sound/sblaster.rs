//! Sound Blaster group: card type, OPL FM model, and the IO triple.

use gtk::prelude::*;
use gtk::{Box as GtkBox, DropDown};

use super::super::rows::{cfg_opt, config_row, heading, Ctx, DEFAULT};
use crate::config::dosbox_conf::DosboxConfig;

const SBTYPE_OPTS: [&str; 9] = [
    DEFAULT, "gb", "sb1", "sb2", "sbpro1", "sbpro2", "sb16", "ess", "none",
];
const OPLMODE_OPTS: [&str; 8] = [
    DEFAULT, "auto", "opl2", "dualopl2", "opl3", "opl3gold", "esfm", "none",
];
const SBBASE_OPTS: [&str; 9] = [
    DEFAULT, "220", "240", "260", "280", "2a0", "2c0", "2e0", "300",
];
const SBIRQ_OPTS: [&str; 8] = [DEFAULT, "3", "5", "7", "9", "10", "11", "12"];
const SBDMA_OPTS: [&str; 7] = [DEFAULT, "0", "1", "3", "5", "6", "7"];

const DEF_SBTYPE: &str = "sb16";
const DEF_OPLMODE: &str = "auto";
const DEF_SBBASE: &str = "220";
const DEF_SBIRQ: &str = "7";
const DEF_SBDMA: &str = "1";
const DEF_SBHDMA: &str = "5";

pub(super) struct Widgets {
    sbtype: DropDown,
    oplmode: DropDown,
    sbbase: DropDown,
    sbirq: DropDown,
    sbdma: DropDown,
    sbhdma: DropDown,
}

pub(super) fn append(page: &GtkBox, config: &DosboxConfig, ctx: &Ctx) -> Widgets {
    page.append(&heading("Sound Blaster"));
    let (row, sbtype) = config_row(
        "Type",
        &SBTYPE_OPTS,
        config.sbtype.as_deref(),
        &ctx.sentinel(DEF_SBTYPE),
    );
    page.append(&row);
    let (row, oplmode) = config_row(
        "OPL mode",
        &OPLMODE_OPTS,
        config.oplmode.as_deref(),
        &ctx.sentinel(DEF_OPLMODE),
    );
    page.append(&row);
    let (row, sbbase) = config_row(
        "Port",
        &SBBASE_OPTS,
        config.sbbase.as_deref(),
        &ctx.sentinel(DEF_SBBASE),
    );
    page.append(&row);
    let (row, sbirq) = config_row(
        "IRQ",
        &SBIRQ_OPTS,
        config.sbirq.as_deref(),
        &ctx.sentinel(DEF_SBIRQ),
    );
    page.append(&row);
    let (row, sbdma) = config_row(
        "DMA",
        &SBDMA_OPTS,
        config.sbdma.as_deref(),
        &ctx.sentinel(DEF_SBDMA),
    );
    page.append(&row);
    let (row, sbhdma) = config_row(
        "High DMA",
        &SBDMA_OPTS,
        config.sbhdma.as_deref(),
        &ctx.sentinel(DEF_SBHDMA),
    );
    page.append(&row);

    Widgets {
        sbtype,
        oplmode,
        sbbase,
        sbirq,
        sbdma,
        sbhdma,
    }
}

impl Widgets {
    pub(super) fn apply(&self, cfg: &mut DosboxConfig) {
        cfg.sbtype = cfg_opt(&self.sbtype);
        cfg.oplmode = cfg_opt(&self.oplmode);
        cfg.sbbase = cfg_opt(&self.sbbase);
        cfg.sbirq = cfg_opt(&self.sbirq);
        cfg.sbdma = cfg_opt(&self.sbdma);
        cfg.sbhdma = cfg_opt(&self.sbhdma);
    }
}
