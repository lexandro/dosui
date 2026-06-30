//! Map known `dosbox.conf` keys to typed [`DosboxConfig`] fields; everything
//! else is kept verbatim as passthrough.

use indexmap::IndexMap;

use crate::config::dosbox_conf::DosboxConfig;

/// Pull the modeled keys out of `sections` into a typed config; the remainder
/// (any section/key we don't model) is preserved as passthrough.
pub(super) fn parse_dosbox(
    mut sections: IndexMap<String, IndexMap<String, String>>,
) -> DosboxConfig {
    let mut cfg = DosboxConfig::default();

    // Pull a known key out of its section (so it doesn't also land in passthrough).
    let mut take = |section: &str, key: &str| -> Option<String> {
        sections.get_mut(section).and_then(|m| m.shift_remove(key))
    };

    cfg.output = take("sdl", "output");
    cfg.fullscreen = take("sdl", "fullscreen").map(|v| parse_bool(&v));
    cfg.vsync = take("sdl", "vsync");
    cfg.machine = take("dosbox", "machine");
    cfg.memsize = take("dosbox", "memsize").and_then(|v| v.parse().ok());
    cfg.vmemsize = take("dosbox", "vmemsize");
    cfg.xms = take("dos", "xms").map(|v| parse_bool(&v));
    cfg.ems = take("dos", "ems");
    cfg.umb = take("dos", "umb").map(|v| parse_bool(&v));
    cfg.core = take("cpu", "core");
    cfg.cputype = take("cpu", "cputype");
    cfg.cycles = take("cpu", "cycles");
    cfg.aspect = take("render", "aspect").map(|v| parse_bool(&v));
    cfg.glshader = take("render", "glshader");
    take("render", "scaler"); // obsolete in dosbox-staging — drop, don't preserve
    cfg.sbtype = take("sblaster", "sbtype");
    cfg.oplmode = take("sblaster", "oplmode");
    cfg.rate = take("mixer", "rate").and_then(|v| v.parse().ok());
    cfg.mididevice = take("midi", "mididevice");
    cfg.mpu401 = take("midi", "mpu401");
    cfg.soundfont = take("fluidsynth", "soundfont");
    cfg.pcspeaker = take("speaker", "pcspeaker");
    cfg.tandy = take("speaker", "tandy");
    cfg.keyboardlayout = take("dos", "keyboardlayout");
    cfg.mouse_capture = take("mouse", "mouse_capture");
    cfg.mouse_sensitivity = take("mouse", "mouse_sensitivity");
    cfg.joysticktype = take("joystick", "joysticktype");
    cfg.joy_autofire = take("joystick", "autofire").map(|v| parse_bool(&v));
    cfg.joy_swap34 = take("joystick", "swap34").map(|v| parse_bool(&v));
    cfg.dos_ver = take("dos", "ver");
    cfg.country = take("dos", "country");

    // Remaining keys -> passthrough (drop now-empty sections).
    sections.retain(|_, keys| !keys.is_empty());
    cfg.passthrough = sections;
    cfg
}

fn parse_bool(value: &str) -> bool {
    matches!(value.to_lowercase().as_str(), "true" | "on" | "1" | "yes")
}
