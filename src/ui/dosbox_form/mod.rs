//! Reusable DOSBox-settings form: the CPU / Graphics / Sound / Advanced tabs.
//!
//! Builds notebook pages from a [`DosboxConfig`] and reads them back with
//! [`DosboxForm::collect`]. Shared by the profile editor (per-profile overrides)
//! and the settings dialog (global defaults). Each tab lives in its own module
//! (`cpu`/`graphics`/`sound`/`advanced`) and exposes a `build` + an `apply` that
//! writes its fields into a [`DosboxConfig`]; this module only wires them up.
//! Shared row helpers and the unset-sentinel presentation live in `rows`.

mod advanced;
mod cpu;
mod graphics;
mod input;
mod memory;
mod rows;
mod sound;

use gtk::prelude::*;
use gtk::Box as GtkBox;

use crate::config::dosbox_conf::DosboxConfig;
use rows::Ctx;

/// The DOSBox tab pages plus the per-tab widgets read on save.
pub struct DosboxForm {
    pub cpu_page: GtkBox,
    pub memory_page: GtkBox,
    pub graphics_page: GtkBox,
    pub sound_page: GtkBox,
    pub input_page: GtkBox,
    pub advanced_page: GtkBox,

    cpu: cpu::Widgets,
    memory: memory::Widgets,
    graphics: graphics::Widgets,
    sound: sound::Widgets,
    input: input::Widgets,
    advanced: advanced::Widgets,
}

impl DosboxForm {
    /// Build the tabs from `config`. `default_label` is the sentinel text shown
    /// for unset values ("(default)" for global defaults, "(inherit)" for a
    /// profile). When `show_builtin` is set, the DOSBox built-in value is shown
    /// next to the sentinel / as a placeholder (used by the global Settings).
    pub fn new(config: &DosboxConfig, default_label: &str, show_builtin: bool) -> DosboxForm {
        let ctx = Ctx {
            label: default_label.to_string(),
            show_builtin,
        };

        let (cpu_page, cpu) = cpu::build(config, &ctx);
        let (memory_page, memory) = memory::build(config, &ctx);
        let (graphics_page, graphics) = graphics::build(config, &ctx);
        let (sound_page, sound) = sound::build(config, &ctx);
        let (input_page, input) = input::build(config, &ctx);
        let (advanced_page, advanced) = advanced::build(config);

        DosboxForm {
            cpu_page,
            memory_page,
            graphics_page,
            sound_page,
            input_page,
            advanced_page,
            cpu,
            memory,
            graphics,
            sound,
            input,
            advanced,
        }
    }

    /// Read the tabs into a [`DosboxConfig`].
    pub fn collect(&self) -> DosboxConfig {
        let mut cfg = DosboxConfig::default();
        self.cpu.apply(&mut cfg);
        self.memory.apply(&mut cfg);
        self.graphics.apply(&mut cfg);
        self.sound.apply(&mut cfg);
        self.input.apply(&mut cfg);
        self.advanced.apply(&mut cfg);
        cfg
    }

    /// Set the read-only preview text (the caller decides what to render).
    pub fn set_preview(&self, text: &str) {
        self.advanced.preview.buffer().set_text(text);
    }
}
