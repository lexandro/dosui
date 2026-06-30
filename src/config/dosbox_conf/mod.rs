//! Typed DOSBox config + `dosbox.conf` (INI) generation.
//!
//! [`DosboxConfig`] models the handful of keys the GUI exposes. Every leaf is
//! `Option`: `None` means "don't emit the key" so dosbox-staging uses its own
//! default. Anything not modeled goes in [`DosboxConfig::passthrough`], an
//! order-preserving `section -> (key -> value)` map — this gives D-Fend-style
//! depth without modeling hundreds of keys.
//!
//! Enumerable-but-open fields (cycles, glshader, …) are stored as `String`: the UI
//! offers known values via dropdowns, but new dosbox-staging values still work.
//!
//! The generated file is an output artifact, regenerated on every launch.
//!
//! This module holds the data model; [`merge`] is the inheritance layer and
//! [`render`] turns a config into INI text.

mod merge;
mod render;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Effective DOSBox settings to render. `None` leaves are omitted from the file.
///
/// The merge target for inheritance: `effective = defaults.merge(profile)`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DosboxConfig {
    /// `[sdl] output` — texture / opengl / …
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    /// `[sdl] fullscreen` — start directly in fullscreen
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fullscreen: Option<bool>,
    /// `[sdl] vsync` — auto / on / adaptive / off
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vsync: Option<String>,
    /// `[dosbox] machine` — svga_s3 / vgaonly / …
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub machine: Option<String>,
    /// `[dosbox] memsize` (MB)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memsize: Option<u32>,
    /// `[dosbox] vmemsize` — video memory (auto / 1 / 2 / 4 / 8 MB)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vmemsize: Option<String>,
    /// `[dos] xms` — Extended Memory
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub xms: Option<bool>,
    /// `[dos] ems` — Expanded Memory (true / emsboard / emm386 / false)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ems: Option<String>,
    /// `[dos] umb` — Upper Memory Blocks
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub umb: Option<bool>,
    /// `[cpu] core` — auto / normal / dynamic / …
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub core: Option<String>,
    /// `[cpu] cputype` — auto / 386 / pentium_slow / …
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cputype: Option<String>,
    /// `[cpu] cycles` — auto / max / "fixed 3000" / …
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cycles: Option<String>,
    /// `[render] aspect`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aspect: Option<bool>,
    /// `[render] glshader` — CRT/GLSL shader (crt-auto / sharp / none / …).
    /// Replaces the obsolete `scaler` key, which dosbox-staging no longer reads.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub glshader: Option<String>,
    /// `[sblaster] sbtype` — sb16 / sbpro2 / none / …
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sbtype: Option<String>,
    /// `[sblaster] oplmode` — OPL FM model (auto / opl2 / opl3 / esfm / none)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oplmode: Option<String>,
    /// `[sblaster] sbbase` — IO port (220, 240, …)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sbbase: Option<String>,
    /// `[sblaster] irq`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sbirq: Option<String>,
    /// `[sblaster] dma`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sbdma: Option<String>,
    /// `[sblaster] hdma` — high DMA (SB16)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sbhdma: Option<String>,
    /// `[mixer] rate` (Hz)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate: Option<u32>,
    /// `[gus] gus` — enable Gravis UltraSound
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gus: Option<bool>,
    /// `[gus] gusbase` — IO port
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gusbase: Option<String>,
    /// `[gus] gusirq`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gusirq: Option<String>,
    /// `[gus] gusdma`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gusdma: Option<String>,
    /// `[midi] mididevice` — auto / mt32 / fluidsynth / none
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mididevice: Option<String>,
    /// `[midi] mpu401` — intelligent / uart / none
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mpu401: Option<String>,
    /// `[fluidsynth] soundfont` — path to a `.sf2` for General MIDI
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub soundfont: Option<String>,
    /// `[speaker] pcspeaker` — PC speaker model (impulse / discrete / none)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pcspeaker: Option<String>,
    /// `[speaker] tandy` — Tandy/PCjr 3-voice sound (auto / on / off)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tandy: Option<String>,
    /// `[dos] keyboardlayout` — auto / us / uk / de / hu / …
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyboardlayout: Option<String>,
    /// `[mouse] mouse_capture` — onclick / onstart / seamless / nomouse
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mouse_capture: Option<String>,
    /// `[mouse] mouse_sensitivity` — percent (e.g. 100)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mouse_sensitivity: Option<String>,
    /// `[joystick] joysticktype` — auto / 2axis / 4axis / fcs / ch / disabled / …
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub joysticktype: Option<String>,
    /// `[joystick] autofire`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub joy_autofire: Option<bool>,
    /// `[joystick] swap34` — swap buttons 3 and 4
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub joy_swap34: Option<bool>,
    /// `[dos] ver` — reported DOS version (3.3 / 5.0 / 6.22 / 7.1)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dos_ver: Option<String>,
    /// `[dos] country` — DOS country code (auto / numeric)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,

    /// Advanced / unmodeled keys: section -> (key -> value), order preserved.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub passthrough: IndexMap<String, IndexMap<String, String>>,
}
