//! Sound tab: one notebook page assembled from device groups, each in its own
//! module (`sblaster`/`gus`/`midi`/`speaker`/`mixer`). Each group appends its
//! rows to the shared page and writes its fields back via `apply`.

mod gus;
mod midi;
mod mixer;
mod sblaster;
mod speaker;

use gtk::Box as GtkBox;

use super::rows::Ctx;
use crate::config::dosbox_conf::DosboxConfig;
use crate::ui::widgets;

/// Sound-tab widgets, read back by [`Widgets::apply`].
pub(super) struct Widgets {
    sblaster: sblaster::Widgets,
    gus: gus::Widgets,
    midi: midi::Widgets,
    speaker: speaker::Widgets,
    mixer: mixer::Widgets,
}

/// Build the Sound page and its read-back widgets.
pub(super) fn build(config: &DosboxConfig, ctx: &Ctx) -> (GtkBox, Widgets) {
    let page = widgets::page();
    let sblaster = sblaster::append(&page, config, ctx);
    let gus = gus::append(&page, config, ctx);
    let midi = midi::append(&page, config, ctx);
    let speaker = speaker::append(&page, config, ctx);
    let mixer = mixer::append(&page, config, ctx);
    (
        page,
        Widgets {
            sblaster,
            gus,
            midi,
            speaker,
            mixer,
        },
    )
}

impl Widgets {
    /// Write the Sound fields into `cfg`.
    pub(super) fn apply(&self, cfg: &mut DosboxConfig) {
        self.sblaster.apply(cfg);
        self.gus.apply(cfg);
        self.midi.apply(cfg);
        self.speaker.apply(cfg);
        self.mixer.apply(cfg);
    }
}
