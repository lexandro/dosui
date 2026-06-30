//! Mixer group: the output sample rate.

use gtk::prelude::*;
use gtk::{Box as GtkBox, Entry};

use super::super::rows::{heading, Ctx};
use crate::config::dosbox_conf::DosboxConfig;
use crate::ui::widgets;

const RATE_PRESETS: [&str; 5] = ["22050", "32000", "44100", "48000", "49716"];
const DEF_RATE: &str = "48000";

pub(super) struct Widgets {
    rate: Entry,
}

pub(super) fn append(page: &GtkBox, config: &DosboxConfig, ctx: &Ctx) -> Widgets {
    page.append(&heading("Mixer"));
    let rate_cur = config.rate.map(|v| v.to_string()).unwrap_or_default();
    let (row, rate) = widgets::combo_row(
        "Rate (Hz)",
        &RATE_PRESETS,
        &rate_cur,
        &ctx.placeholder(DEF_RATE),
    );
    page.append(&row);

    Widgets { rate }
}

impl Widgets {
    pub(super) fn apply(&self, cfg: &mut DosboxConfig) {
        cfg.rate = widgets::none_if_empty(&self.rate.text()).and_then(|s| s.parse().ok());
    }
}
