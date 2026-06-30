//! MIDI group: device, MPU-401 mode, and the FluidSynth SoundFont path.

use gtk::prelude::*;
use gtk::{Box as GtkBox, DropDown, Entry};

use super::super::rows::{cfg_opt, config_row, heading, Ctx, DEFAULT};
use crate::config::dosbox_conf::DosboxConfig;
use crate::ui::widgets;

const MIDI_OPTS: [&str; 5] = [DEFAULT, "auto", "mt32", "fluidsynth", "none"];
const MPU401_OPTS: [&str; 4] = [DEFAULT, "intelligent", "uart", "none"];

const DEF_MIDI: &str = "auto";
const DEF_MPU401: &str = "intelligent";

pub(super) struct Widgets {
    mididevice: DropDown,
    mpu401: DropDown,
    soundfont: Entry,
}

pub(super) fn append(page: &GtkBox, config: &DosboxConfig, ctx: &Ctx) -> Widgets {
    page.append(&heading("MIDI"));
    let (row, mididevice) = config_row(
        "Device",
        &MIDI_OPTS,
        config.mididevice.as_deref(),
        &ctx.sentinel(DEF_MIDI),
    );
    page.append(&row);
    let (row, mpu401) = config_row(
        "MPU-401",
        &MPU401_OPTS,
        config.mpu401.as_deref(),
        &ctx.sentinel(DEF_MPU401),
    );
    page.append(&row);
    let (row, soundfont, browse) =
        widgets::file_row("SoundFont (.sf2)", widgets::opt(&config.soundfont));
    widgets::wire_browse_root(&soundfont, &browse, "Select a SoundFont");
    page.append(&row);

    Widgets {
        mididevice,
        mpu401,
        soundfont,
    }
}

impl Widgets {
    pub(super) fn apply(&self, cfg: &mut DosboxConfig) {
        cfg.mididevice = cfg_opt(&self.mididevice);
        cfg.mpu401 = cfg_opt(&self.mpu401);
        cfg.soundfont = widgets::none_if_empty(&self.soundfont.text());
    }
}
