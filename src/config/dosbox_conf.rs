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
//! Over the 150-line soft cap by design: the config model + INI rendering +
//! inheritance merge, plus their inline tests.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::profile::{Mount, MountKind, RunSpec};

/// Effective DOSBox settings to render. `None` leaves are omitted from the file.
///
/// In M4 this is also the merge target: `effective = merge(defaults, overrides)`.
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

    /// Advanced / unmodeled keys: section -> (key -> value), order preserved.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub passthrough: IndexMap<String, IndexMap<String, String>>,
}

impl DosboxConfig {
    /// Layer `overrides` on top of `self` (the defaults): every set leaf in
    /// `overrides` wins; unset (`None`) leaves inherit from `self`. Passthrough
    /// maps merge per (section, key), with `overrides` winning.
    ///
    /// This is the inheritance model: `effective = defaults.merge(profile)`.
    pub fn merge(&self, overrides: &DosboxConfig) -> DosboxConfig {
        let pick = |o: &Option<String>, d: &Option<String>| o.clone().or_else(|| d.clone());

        let mut passthrough = self.passthrough.clone();
        for (section, keys) in &overrides.passthrough {
            let target = passthrough.entry(section.clone()).or_default();
            for (key, value) in keys {
                target.insert(key.clone(), value.clone());
            }
        }

        DosboxConfig {
            output: pick(&overrides.output, &self.output),
            fullscreen: overrides.fullscreen.or(self.fullscreen),
            vsync: pick(&overrides.vsync, &self.vsync),
            machine: pick(&overrides.machine, &self.machine),
            memsize: overrides.memsize.or(self.memsize),
            vmemsize: pick(&overrides.vmemsize, &self.vmemsize),
            xms: overrides.xms.or(self.xms),
            ems: pick(&overrides.ems, &self.ems),
            umb: overrides.umb.or(self.umb),
            core: pick(&overrides.core, &self.core),
            cputype: pick(&overrides.cputype, &self.cputype),
            cycles: pick(&overrides.cycles, &self.cycles),
            aspect: overrides.aspect.or(self.aspect),
            glshader: pick(&overrides.glshader, &self.glshader),
            sbtype: pick(&overrides.sbtype, &self.sbtype),
            sbbase: pick(&overrides.sbbase, &self.sbbase),
            sbirq: pick(&overrides.sbirq, &self.sbirq),
            sbdma: pick(&overrides.sbdma, &self.sbdma),
            sbhdma: pick(&overrides.sbhdma, &self.sbhdma),
            rate: overrides.rate.or(self.rate),
            gus: overrides.gus.or(self.gus),
            gusbase: pick(&overrides.gusbase, &self.gusbase),
            gusirq: pick(&overrides.gusirq, &self.gusirq),
            gusdma: pick(&overrides.gusdma, &self.gusdma),
            mididevice: pick(&overrides.mididevice, &self.mididevice),
            passthrough,
        }
    }

    /// Render a full `dosbox.conf`: typed sections, then passthrough, then the
    /// `[autoexec]` block built from `run`.
    pub fn render(&self, run: &RunSpec) -> String {
        let mut sections: IndexMap<&str, IndexMap<String, String>> = IndexMap::new();

        // Typed keys, grouped into a stable section order.
        if let Some(v) = &self.output {
            put(&mut sections, "sdl", "output", v.clone());
        }
        if let Some(v) = self.fullscreen {
            put(&mut sections, "sdl", "fullscreen", bool_str(v));
        }
        if let Some(v) = &self.vsync {
            put(&mut sections, "sdl", "vsync", v.clone());
        }
        if let Some(v) = &self.machine {
            put(&mut sections, "dosbox", "machine", v.clone());
        }
        if let Some(v) = self.memsize {
            put(&mut sections, "dosbox", "memsize", v.to_string());
        }
        if let Some(v) = &self.vmemsize {
            put(&mut sections, "dosbox", "vmemsize", v.clone());
        }
        if let Some(v) = self.xms {
            put(&mut sections, "dos", "xms", bool_str(v));
        }
        if let Some(v) = &self.ems {
            put(&mut sections, "dos", "ems", v.clone());
        }
        if let Some(v) = self.umb {
            put(&mut sections, "dos", "umb", bool_str(v));
        }
        if let Some(v) = &self.core {
            put(&mut sections, "cpu", "core", v.clone());
        }
        if let Some(v) = &self.cputype {
            put(&mut sections, "cpu", "cputype", v.clone());
        }
        if let Some(v) = &self.cycles {
            put(&mut sections, "cpu", "cycles", v.clone());
        }
        if let Some(v) = self.aspect {
            put(&mut sections, "render", "aspect", bool_str(v));
        }
        if let Some(v) = &self.glshader {
            put(&mut sections, "render", "glshader", v.clone());
        }
        if let Some(v) = &self.rate {
            put(&mut sections, "mixer", "rate", v.to_string());
        }
        if let Some(v) = &self.sbtype {
            put(&mut sections, "sblaster", "sbtype", v.clone());
        }
        if let Some(v) = &self.sbbase {
            put(&mut sections, "sblaster", "sbbase", v.clone());
        }
        if let Some(v) = &self.sbirq {
            put(&mut sections, "sblaster", "irq", v.clone());
        }
        if let Some(v) = &self.sbdma {
            put(&mut sections, "sblaster", "dma", v.clone());
        }
        if let Some(v) = &self.sbhdma {
            put(&mut sections, "sblaster", "hdma", v.clone());
        }
        if let Some(v) = self.gus {
            put(&mut sections, "gus", "gus", bool_str(v));
        }
        if let Some(v) = &self.gusbase {
            put(&mut sections, "gus", "gusbase", v.clone());
        }
        if let Some(v) = &self.gusirq {
            put(&mut sections, "gus", "gusirq", v.clone());
        }
        if let Some(v) = &self.gusdma {
            put(&mut sections, "gus", "gusdma", v.clone());
        }
        if let Some(v) = &self.mididevice {
            put(&mut sections, "midi", "mididevice", v.clone());
        }

        // Passthrough merges underneath; typed keys already set above win.
        for (section, keys) in &self.passthrough {
            for (key, value) in keys {
                sections
                    .entry(section.as_str())
                    .or_default()
                    .entry(key.clone())
                    .or_insert_with(|| value.clone());
            }
        }

        let mut out = String::new();
        for (section, keys) in &sections {
            out.push_str(&format!("[{section}]\n"));
            for (key, value) in keys {
                out.push_str(&format!("{key} = {value}\n"));
            }
            out.push('\n');
        }

        out.push_str("[autoexec]\n");
        for line in autoexec_lines(run) {
            out.push_str(&line);
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

fn put<'a>(
    sections: &mut IndexMap<&'a str, IndexMap<String, String>>,
    section: &'a str,
    key: &str,
    value: String,
) {
    sections
        .entry(section)
        .or_default()
        .insert(key.to_string(), value);
}

fn bool_str(b: bool) -> String {
    if b { "true" } else { "false" }.to_string()
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
    fn merge_overrides_win_and_unset_inherit() {
        let defaults = DosboxConfig {
            cycles: Some("auto".into()),
            memsize: Some(16),
            output: Some("opengl".into()),
            ..Default::default()
        };
        let overrides = DosboxConfig {
            cycles: Some("max".into()), // wins
            memsize: None,              // inherits 16
            ..Default::default()
        };
        let effective = defaults.merge(&overrides);
        assert_eq!(effective.cycles.as_deref(), Some("max"));
        assert_eq!(effective.memsize, Some(16));
        assert_eq!(effective.output.as_deref(), Some("opengl"));
    }

    #[test]
    fn merge_passthrough_combines_with_override_winning() {
        let mut defaults = DosboxConfig::default();
        let mut d = IndexMap::new();
        d.insert("glshader".to_string(), "crt".to_string());
        d.insert("aspect".to_string(), "true".to_string());
        defaults.passthrough.insert("render".to_string(), d);

        let mut overrides = DosboxConfig::default();
        let mut o = IndexMap::new();
        o.insert("glshader".to_string(), "sharp".to_string()); // wins
        overrides.passthrough.insert("render".to_string(), o);

        let effective = defaults.merge(&overrides);
        assert_eq!(effective.passthrough["render"]["glshader"], "sharp");
        assert_eq!(effective.passthrough["render"]["aspect"], "true");
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
