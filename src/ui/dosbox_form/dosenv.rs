//! DOS environment tab: reported DOS version and country code.

use gtk::prelude::*;
use gtk::{Box as GtkBox, Entry};

use super::rows::Ctx;
use crate::config::dosbox_conf::DosboxConfig;
use crate::ui::widgets;

const VER_PRESETS: [&str; 4] = ["3.3", "5.0", "6.22", "7.1"];

const DEF_VER: &str = "5.0";
const DEF_COUNTRY: &str = "auto";

/// DOS-environment-tab widgets, read back by [`Widgets::apply`].
pub(super) struct Widgets {
    dos_ver: Entry,
    country: Entry,
}

/// Build the DOS environment page and its read-back widgets.
pub(super) fn build(config: &DosboxConfig, ctx: &Ctx) -> (GtkBox, Widgets) {
    let page = widgets::page();

    let (row, dos_ver) = widgets::combo_row(
        "DOS version",
        &VER_PRESETS,
        widgets::opt(&config.dos_ver),
        &ctx.placeholder(DEF_VER),
    );
    page.append(&row);
    let (row, country) = widgets::entry_row("Country code", widgets::opt(&config.country));
    country.set_placeholder_text(Some(&ctx.placeholder(DEF_COUNTRY)));
    page.append(&row);

    (page, Widgets { dos_ver, country })
}

impl Widgets {
    /// Write the DOS environment fields into `cfg`.
    pub(super) fn apply(&self, cfg: &mut DosboxConfig) {
        cfg.dos_ver = widgets::none_if_empty(&self.dos_ver.text());
        cfg.country = widgets::none_if_empty(&self.country.text());
    }
}
