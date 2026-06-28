//! Typed DOSBox config + `dosbox.conf` (INI) generation.
//!
//! [`DosboxConfig`] models the handful of keys the GUI exposes. Every leaf is
//! `Option`: `None` means "don't emit the key" so dosbox-staging uses its own
//! default. Anything not modeled goes in [`DosboxConfig::passthrough`], an
//! order-preserving `section -> (key -> value)` map — this gives D-Fend-style
//! depth without modeling hundreds of keys.
//!
//! Enumerable-but-open fields (cycles, scaler, …) are stored as `String`: the UI
//! offers known values via dropdowns, but new dosbox-staging values still work.
//!
//! The generated file is an output artifact, regenerated on every launch.

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
    /// `[dosbox] machine` — svga_s3 / vgaonly / …
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub machine: Option<String>,
    /// `[dosbox] memsize` (MB)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memsize: Option<u32>,
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
    /// `[render] scaler` — none / normal2x / …
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scaler: Option<String>,
    /// `[sblaster] sbtype` — sb16 / sbpro2 / none / …
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sbtype: Option<String>,
    /// `[mixer] rate` (Hz)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate: Option<u32>,

    /// Advanced / unmodeled keys: section -> (key -> value), order preserved.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub passthrough: IndexMap<String, IndexMap<String, String>>,
}

impl DosboxConfig {
    /// Render a full `dosbox.conf`: typed sections, then passthrough, then the
    /// `[autoexec]` block built from `run`.
    pub fn render(&self, run: &RunSpec) -> String {
        let mut sections: IndexMap<&str, IndexMap<String, String>> = IndexMap::new();

        // Typed keys, grouped into a stable section order.
        if let Some(v) = &self.output {
            put(&mut sections, "sdl", "output", v.clone());
        }
        if let Some(v) = &self.machine {
            put(&mut sections, "dosbox", "machine", v.clone());
        }
        if let Some(v) = self.memsize {
            put(&mut sections, "dosbox", "memsize", v.to_string());
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
        if let Some(v) = &self.scaler {
            put(&mut sections, "render", "scaler", v.clone());
        }
        if let Some(v) = &self.rate {
            put(&mut sections, "mixer", "rate", v.to_string());
        }
        if let Some(v) = &self.sbtype {
            put(&mut sections, "sblaster", "sbtype", v.clone());
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
fn autoexec_lines(run: &RunSpec) -> Vec<String> {
    let mut lines = Vec::new();

    for m in &run.mounts {
        lines.push(mount_line(m));
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
