//! Sound tab: Sound Blaster, Gravis UltraSound, mixer rate, MIDI device.

use gtk::prelude::*;
use gtk::{Box as GtkBox, DropDown, Entry};

use super::rows::{bool_opt, cfg_bool, cfg_opt, config_row, heading, Ctx, DEFAULT};
use crate::config::dosbox_conf::DosboxConfig;
use crate::ui::widgets;

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
const RATE_PRESETS: [&str; 5] = ["22050", "32000", "44100", "48000", "49716"];

const DEF_SBTYPE: &str = "sb16";
const DEF_SBBASE: &str = "220";
const DEF_SBIRQ: &str = "7";
const DEF_SBDMA: &str = "1";
const DEF_SBHDMA: &str = "5";
const DEF_GUS: &str = "off";
const DEF_GUSBASE: &str = "240";
const DEF_GUSIRQ: &str = "5";
const DEF_GUSDMA: &str = "3";
const DEF_RATE: &str = "48000";
const DEF_MIDI: &str = "auto";

/// Sound-tab widgets, read back by [`Widgets::apply`].
pub(super) struct Widgets {
    sbtype: DropDown,
    sbbase: DropDown,
    sbirq: DropDown,
    sbdma: DropDown,
    sbhdma: DropDown,
    gus: DropDown,
    gusbase: DropDown,
    gusirq: DropDown,
    gusdma: DropDown,
    rate: Entry,
    mididevice: DropDown,
}

/// Build the Sound page and its read-back widgets.
pub(super) fn build(config: &DosboxConfig, ctx: &Ctx) -> (GtkBox, Widgets) {
    let page = widgets::page();

    page.append(&heading("Sound Blaster"));
    let (row, sbtype) = config_row(
        "Type",
        &SBTYPE_OPTS,
        config.sbtype.as_deref(),
        &ctx.sentinel(DEF_SBTYPE),
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

    page.append(&heading("Mixer"));
    let rate_cur = config.rate.map(|v| v.to_string()).unwrap_or_default();
    let (row, rate) = widgets::combo_row(
        "Rate (Hz)",
        &RATE_PRESETS,
        &rate_cur,
        &ctx.placeholder(DEF_RATE),
    );
    page.append(&row);

    page.append(&heading("MIDI"));
    let (row, mididevice) = config_row(
        "Device",
        &MIDI_OPTS,
        config.mididevice.as_deref(),
        &ctx.sentinel(DEF_MIDI),
    );
    page.append(&row);

    (
        page,
        Widgets {
            sbtype,
            sbbase,
            sbirq,
            sbdma,
            sbhdma,
            gus,
            gusbase,
            gusirq,
            gusdma,
            rate,
            mididevice,
        },
    )
}

impl Widgets {
    /// Write the Sound fields into `cfg`.
    pub(super) fn apply(&self, cfg: &mut DosboxConfig) {
        cfg.sbtype = cfg_opt(&self.sbtype);
        cfg.sbbase = cfg_opt(&self.sbbase);
        cfg.sbirq = cfg_opt(&self.sbirq);
        cfg.sbdma = cfg_opt(&self.sbdma);
        cfg.sbhdma = cfg_opt(&self.sbhdma);
        cfg.gus = cfg_bool(&self.gus);
        cfg.gusbase = cfg_opt(&self.gusbase);
        cfg.gusirq = cfg_opt(&self.gusirq);
        cfg.gusdma = cfg_opt(&self.gusdma);
        cfg.rate = widgets::none_if_empty(&self.rate.text()).and_then(|s| s.parse().ok());
        cfg.mididevice = cfg_opt(&self.mididevice);
    }
}
