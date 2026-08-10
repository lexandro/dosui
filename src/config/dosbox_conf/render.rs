//! Render a [`DosboxConfig`] to `dosbox.conf` INI text.
//!
//! Over the 150-line soft cap by design: roughly half the file is the inline
//! test suite that pins each section's output, and those belong next to the
//! `ini.*` call order they verify.

use indexmap::IndexMap;

use super::DosboxConfig;
use crate::config::profile::{Mount, MountKind, RunSpec};

impl DosboxConfig {
    /// Render a full `dosbox.conf`: typed sections, then passthrough (which never
    /// overrides a typed key), then the `[autoexec]` block built from `run`.
    pub fn render(&self, run: &RunSpec) -> String {
        let mut ini = Ini::default();
        // Typed keys in a stable section/key order (`opt` = string, `flag` =
        // bool, `num` = integer; `None` leaves are skipped).
        ini.opt("sdl", "output", &self.output);
        ini.flag("sdl", "fullscreen", self.fullscreen);
        ini.opt("sdl", "vsync", &self.vsync);
        ini.opt("dosbox", "machine", &self.machine);
        ini.num("dosbox", "memsize", self.memsize);
        ini.opt("dosbox", "vmemsize", &self.vmemsize);
        ini.flag("dos", "xms", self.xms);
        ini.opt("dos", "ems", &self.ems);
        ini.flag("dos", "umb", self.umb);
        ini.opt("dos", "keyboardlayout", &self.keyboardlayout);
        ini.opt("dos", "ver", &self.dos_ver);
        ini.opt("dos", "country", &self.country);
        ini.opt("mouse", "mouse_capture", &self.mouse_capture);
        ini.opt("mouse", "mouse_sensitivity", &self.mouse_sensitivity);
        ini.opt("joystick", "joysticktype", &self.joysticktype);
        ini.flag("joystick", "autofire", self.joy_autofire);
        ini.flag("joystick", "swap34", self.joy_swap34);
        ini.opt("cpu", "core", &self.core);
        ini.opt("cpu", "cputype", &self.cputype);
        ini.opt("cpu", "cycles", &self.cycles);
        ini.flag("render", "aspect", self.aspect);
        ini.opt("render", "glshader", &self.glshader);
        ini.num("mixer", "rate", self.rate);
        ini.opt("sblaster", "sbtype", &self.sbtype);
        ini.opt("sblaster", "oplmode", &self.oplmode);
        ini.opt("sblaster", "sbbase", &self.sbbase);
        ini.opt("sblaster", "irq", &self.sbirq);
        ini.opt("sblaster", "dma", &self.sbdma);
        ini.opt("sblaster", "hdma", &self.sbhdma);
        ini.flag("gus", "gus", self.gus);
        ini.opt("gus", "gusbase", &self.gusbase);
        ini.opt("gus", "gusirq", &self.gusirq);
        ini.opt("gus", "gusdma", &self.gusdma);
        ini.opt("midi", "mididevice", &self.mididevice);
        ini.opt("midi", "mpu401", &self.mpu401);
        ini.opt("fluidsynth", "soundfont", &self.soundfont);
        ini.opt("speaker", "pcspeaker", &self.pcspeaker);
        ini.opt("speaker", "tandy", &self.tandy);

        // Passthrough merges underneath; typed keys already set above win.
        for (section, keys) in &self.passthrough {
            for (key, value) in keys {
                ini.sections
                    .entry(section.clone())
                    .or_default()
                    .entry(key.clone())
                    .or_insert_with(|| value.clone());
            }
        }

        let mut out = ini.text();
        out.push_str("[autoexec]\n");
        for line in autoexec_lines(run) {
            out.push_str(&line);
            out.push('\n');
        }
        out
    }
}

/// Accumulates `section -> (key -> value)` in first-seen order, then formats it.
#[derive(Default)]
struct Ini {
    sections: IndexMap<String, IndexMap<String, String>>,
}

impl Ini {
    /// Emit an optional string value (skips `None`).
    fn opt(&mut self, section: &str, key: &str, value: &Option<String>) {
        if let Some(v) = value {
            self.put(section, key, v.clone());
        }
    }
    /// Emit an optional bool as `true`/`false` (skips `None`).
    fn flag(&mut self, section: &str, key: &str, value: Option<bool>) {
        if let Some(v) = value {
            self.put(section, key, if v { "true" } else { "false" }.to_string());
        }
    }
    /// Emit an optional integer (skips `None`).
    fn num(&mut self, section: &str, key: &str, value: Option<u32>) {
        if let Some(v) = value {
            self.put(section, key, v.to_string());
        }
    }
    fn put(&mut self, section: &str, key: &str, value: String) {
        self.sections
            .entry(section.to_string())
            .or_default()
            .insert(key.to_string(), value);
    }
    /// Format the accumulated sections as INI (a trailing blank line per section).
    fn text(&self) -> String {
        let mut out = String::new();
        for (section, keys) in &self.sections {
            out.push_str(&format!("[{section}]\n"));
            for (key, value) in keys {
                out.push_str(&format!("{key} = {value}\n"));
            }
            out.push('\n');
        }
        out
    }
}

/// Build the `[autoexec]` lines: mounts, switch drive, run command, optional exit.
///
/// An empty command means "DOS console": mount the drives, switch into the
/// working drive (only if it was mounted), and stay at the prompt — no exit — so
/// the user can run things by hand.
fn autoexec_lines(run: &RunSpec) -> Vec<String> {
    let mut lines = Vec::new();
    for m in &run.mounts {
        lines.push(mount_line(m));
    }

    let working_mounted = run.mounts.iter().any(|m| m.drive == run.working_drive);

    if run.command.trim().is_empty() {
        if working_mounted {
            lines.push(format!("{}:", run.working_drive));
        }
        return lines; // console: drop to the prompt, no command, no exit
    }

    lines.push(format!("{}:", run.working_drive));
    let mut command = run.command.clone();
    if !run.args.is_empty() {
        command.push(' ');
        command.push_str(&run.args.join(" "));
    }
    lines.push(command);
    if run.exit_after {
        lines.push("exit".to_string());
    }
    lines
}

/// One `mount`/`imgmount` line for a drive. Paths are always quoted.
fn mount_line(m: &Mount) -> String {
    let path = m.path.display();
    match m.kind {
        MountKind::Directory => {
            let label = m
                .label
                .as_deref()
                .map(|l| format!(" -label {l}"))
                .unwrap_or_default();
            format!("mount {} \"{}\"{}", m.drive, path, label)
        }
        MountKind::CdImage => format!("imgmount {} \"{}\" -t cdrom", m.drive, path),
        MountKind::FloppyImage => format!("imgmount {} \"{}\" -t floppy", m.drive, path),
        MountKind::HddImage => format!("imgmount {} \"{}\" -t hdd", m.drive, path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(exit_after: bool) -> RunSpec {
        RunSpec {
            mounts: vec![Mount {
                drive: 'C',
                kind: MountKind::Directory,
                path: "/games/dune 2".into(),
                label: None,
            }],
            working_drive: 'C',
            command: "DUNE2.EXE".into(),
            args: vec![],
            exit_after,
        }
    }

    #[test]
    fn empty_config_emits_only_autoexec() {
        let conf = DosboxConfig::default().render(&run(true));
        assert!(!conf.contains("[cpu]"));
        assert_eq!(
            conf,
            "[autoexec]\nmount C \"/games/dune 2\"\nC:\nDUNE2.EXE\nexit\n"
        );
    }

    #[test]
    fn empty_command_opens_console() {
        // With a mount: switch into it, stay at the prompt (no command, no exit).
        let mut r = run(true);
        r.command = String::new();
        let conf = DosboxConfig::default().render(&r);
        assert_eq!(conf, "[autoexec]\nmount C \"/games/dune 2\"\nC:\n");

        // No mounts: just the bare Z:\ prompt (no drive switch).
        let bare = RunSpec {
            mounts: vec![],
            working_drive: 'C',
            command: String::new(),
            args: vec![],
            exit_after: true,
        };
        assert_eq!(DosboxConfig::default().render(&bare), "[autoexec]\n");
    }

    #[test]
    fn typed_keys_render_into_their_sections() {
        let conf = DosboxConfig {
            memsize: Some(16),
            cycles: Some("fixed 3000".into()),
            output: Some("opengl".into()),
            aspect: Some(true),
            ..Default::default()
        }
        .render(&run(false));

        assert!(conf.contains("[sdl]\noutput = opengl\n"));
        assert!(conf.contains("[dosbox]\nmemsize = 16\n"));
        assert!(conf.contains("[cpu]\ncycles = fixed 3000\n"));
        assert!(conf.contains("[render]\naspect = true\n"));
        // exit_after=false -> no trailing exit
        assert!(!conf.contains("\nexit\n"));
    }

    #[test]
    fn display_keys_render_into_sdl_and_render() {
        let conf = DosboxConfig {
            fullscreen: Some(true),
            vsync: Some("on".into()),
            glshader: Some("sharp".into()),
            ..Default::default()
        }
        .render(&run(false));
        assert!(conf.contains("fullscreen = true\n"));
        assert!(conf.contains("vsync = on\n"));
        assert!(conf.contains("[render]\nglshader = sharp\n"));
    }

    #[test]
    fn memory_keys_render_into_dosbox_and_dos() {
        let conf = DosboxConfig {
            vmemsize: Some("4".into()),
            xms: Some(true),
            ems: Some("emm386".into()),
            umb: Some(false),
            ..Default::default()
        }
        .render(&run(false));
        assert!(conf.contains("vmemsize = 4\n"));
        assert!(conf.contains("[dos]\n"));
        assert!(conf.contains("xms = true\n"));
        assert!(conf.contains("ems = emm386\n"));
        assert!(conf.contains("umb = false\n"));
    }

    #[test]
    fn extra_sound_keys_render_into_their_sections() {
        let conf = DosboxConfig {
            oplmode: Some("opl3".into()),
            mpu401: Some("uart".into()),
            soundfont: Some("/sf/gm.sf2".into()),
            pcspeaker: Some("discrete".into()),
            tandy: Some("on".into()),
            ..Default::default()
        }
        .render(&run(false));
        assert!(conf.contains("oplmode = opl3\n"));
        assert!(conf.contains("mpu401 = uart\n"));
        assert!(conf.contains("[fluidsynth]\nsoundfont = /sf/gm.sf2\n"));
        assert!(conf.contains("[speaker]\n"));
        assert!(conf.contains("pcspeaker = discrete\n"));
        assert!(conf.contains("tandy = on\n"));
    }

    #[test]
    fn input_keys_render_into_dos_mouse_joystick() {
        let conf = DosboxConfig {
            keyboardlayout: Some("hu".into()),
            mouse_capture: Some("seamless".into()),
            mouse_sensitivity: Some("80".into()),
            joysticktype: Some("4axis".into()),
            joy_autofire: Some(true),
            joy_swap34: Some(false),
            ..Default::default()
        }
        .render(&run(false));
        assert!(conf.contains("keyboardlayout = hu\n"));
        assert!(conf.contains("[mouse]\n"));
        assert!(conf.contains("mouse_capture = seamless\n"));
        assert!(conf.contains("mouse_sensitivity = 80\n"));
        assert!(conf.contains("[joystick]\n"));
        assert!(conf.contains("joysticktype = 4axis\n"));
        assert!(conf.contains("autofire = true\n"));
        assert!(conf.contains("swap34 = false\n"));
    }

    #[test]
    fn dos_env_keys_render_into_dos() {
        let conf = DosboxConfig {
            dos_ver: Some("6.22".into()),
            country: Some("36".into()),
            ..Default::default()
        }
        .render(&run(false));
        assert!(conf.contains("[dos]\n"));
        assert!(conf.contains("ver = 6.22\n"));
        assert!(conf.contains("country = 36\n"));
    }

    #[test]
    fn sound_card_io_renders_into_sblaster_and_gus() {
        let conf = DosboxConfig {
            sbtype: Some("sb16".into()),
            sbbase: Some("240".into()),
            sbirq: Some("5".into()),
            sbdma: Some("1".into()),
            sbhdma: Some("5".into()),
            gus: Some(true),
            gusbase: Some("240".into()),
            gusirq: Some("5".into()),
            gusdma: Some("3".into()),
            ..Default::default()
        }
        .render(&run(false));
        assert!(conf.contains("[sblaster]\n"));
        assert!(conf.contains("sbbase = 240\n"));
        assert!(conf.contains("irq = 5\n"));
        assert!(conf.contains("dma = 1\n"));
        assert!(conf.contains("hdma = 5\n"));
        assert!(conf.contains("[gus]\n"));
        assert!(conf.contains("gus = true\n"));
        assert!(conf.contains("gusbase = 240\n"));
        assert!(conf.contains("gusirq = 5\n"));
        assert!(conf.contains("gusdma = 3\n"));
    }

    #[test]
    fn passthrough_keys_appear_without_overriding_typed() {
        let mut cfg = DosboxConfig {
            cycles: Some("max".into()),
            ..Default::default()
        };
        let mut cpu = IndexMap::new();
        cpu.insert("cycles".to_string(), "auto".to_string()); // must NOT win
        cpu.insert("cycleup".to_string(), "500".to_string());
        cfg.passthrough.insert("cpu".to_string(), cpu);

        let conf = cfg.render(&run(false));
        assert!(conf.contains("cycles = max"));
        assert!(!conf.contains("cycles = auto"));
        assert!(conf.contains("cycleup = 500"));
    }

    #[test]
    fn image_mounts_use_imgmount() {
        let mut r = run(false);
        r.mounts = vec![Mount {
            drive: 'D',
            kind: MountKind::CdImage,
            path: "/iso/game.cue".into(),
            label: None,
        }];
        let conf = DosboxConfig::default().render(&r);
        assert!(conf.contains("imgmount D \"/iso/game.cue\" -t cdrom"));
    }

    #[test]
    fn command_args_are_appended() {
        let mut r = run(false);
        r.command = "SETUP.EXE".into();
        r.args = vec!["/install".into()];
        let conf = DosboxConfig::default().render(&r);
        assert!(conf.contains("SETUP.EXE /install"));
    }
}
