//! Import an existing `dosbox.conf` into a dosui [`Profile`].
//!
//! This is what "import from D-Fend / DBGL" means in practice: both ultimately
//! produce a `dosbox.conf`. We parse the `[autoexec]` block into mounts + a run
//! command ([`autoexec`]), map the keys we model into a typed [`DosboxConfig`]
//! and keep the rest as passthrough ([`keys`]). Best-effort and GTK-free.
//!
//! Over the 150-line soft cap by design: the bulk is the inline round-trip
//! suite, which is the guard that keeps this module in step with `render`.

mod autoexec;
mod keys;

use indexmap::IndexMap;

use super::profile::Profile;

/// Build a profile from `dosbox.conf` text. `title` seeds the display name/id.
pub fn import_profile(text: &str, title: &str) -> Profile {
    let (sections, autoexec_lines) = parse_ini(text);
    Profile {
        id: String::new(),
        title: title.to_string(),
        genre: None,
        year: None,
        developer: None,
        publisher: None,
        www: None,
        notes: None,
        cover: None,
        favorite: false,
        last_played: None,
        run: autoexec::parse_run(&autoexec_lines),
        dosbox: keys::parse_dosbox(sections),
    }
}

/// Split INI text into `section -> key -> value` plus the raw `[autoexec]` lines.
fn parse_ini(text: &str) -> (IndexMap<String, IndexMap<String, String>>, Vec<String>) {
    let mut sections: IndexMap<String, IndexMap<String, String>> = IndexMap::new();
    let mut autoexec: Vec<String> = Vec::new();
    let mut current = String::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            current = name.trim().to_lowercase();
            continue;
        }
        if current == "autoexec" {
            autoexec.push(line.to_string());
            continue;
        }
        // `#` is dosbox's comment marker; `;` is the classic INI one that other
        // writers emit. Neither is a key, so skip both rather than round-tripping
        // "; foo" into passthrough as a bogus key.
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            sections
                .entry(current.clone())
                .or_default()
                .insert(key.trim().to_lowercase(), value.trim().to_string());
        }
    }
    (sections, autoexec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::dosbox_conf::DosboxConfig;
    use crate::config::profile::{Mount, MountKind, RunSpec};

    const SAMPLE: &str = r#"
[sdl]
output = opengl
[cpu]
cycles = 20000
cycleup = 500
[render]
aspect = true
[autoexec]
mount C "/games/Dune 2"
imgmount D "/iso/dune.cue" -t cdrom
C:
DUNE2.EXE /nosound
exit
"#;

    #[test]
    fn imports_run_and_config() {
        let p = import_profile(SAMPLE, "Dune II");
        assert_eq!(p.title, "Dune II");

        // run
        assert_eq!(p.run.working_drive, 'C');
        assert_eq!(p.run.command, "DUNE2.EXE");
        assert_eq!(p.run.args, vec!["/nosound"]);
        assert!(p.run.exit_after);
        assert_eq!(p.run.mounts.len(), 2);
        assert_eq!(p.run.mounts[0].drive, 'C');
        assert_eq!(p.run.mounts[0].kind, MountKind::Directory);
        assert_eq!(p.run.mounts[0].path.to_str().unwrap(), "/games/Dune 2");
        assert_eq!(p.run.mounts[1].kind, MountKind::CdImage);

        // typed config
        assert_eq!(p.dosbox.output.as_deref(), Some("opengl"));
        assert_eq!(p.dosbox.cycles.as_deref(), Some("20000"));
        assert_eq!(p.dosbox.aspect, Some(true));
        // unmodeled key kept as passthrough
        assert_eq!(p.dosbox.passthrough["cpu"]["cycleup"], "500");
    }

    #[test]
    fn mount_label_is_preserved() {
        let p = import_profile("[autoexec]\nmount C \"/games/x\" -label DATA\n", "X");
        assert_eq!(p.run.mounts[0].label.as_deref(), Some("DATA"));
    }

    #[test]
    fn semicolon_comments_are_not_keys() {
        let p = import_profile("[cpu]\n; core = normal\ncputype = 386\n", "X");
        assert_eq!(p.dosbox.cputype.as_deref(), Some("386"));
        assert!(
            p.dosbox.passthrough.is_empty(),
            "comment must not become a key"
        );
    }

    /// Every typed [`DosboxConfig`] leaf set to a distinct non-default value.
    ///
    /// Deliberately an **exhaustive struct literal** (no `..Default::default()`):
    /// adding a field to `DosboxConfig` fails to compile here until the author
    /// decides how it round-trips. That is what makes
    /// [`every_typed_key_survives_render_then_import`] a real guard rather than a
    /// snapshot of whatever was modeled the day it was written.
    fn fully_populated() -> DosboxConfig {
        let mut passthrough = IndexMap::new();
        let mut cpu = IndexMap::new();
        cpu.insert("cycleup".to_string(), "500".to_string());
        passthrough.insert("cpu".to_string(), cpu);

        DosboxConfig {
            output: Some("opengl".into()),
            fullscreen: Some(true),
            vsync: Some("adaptive".into()),
            machine: Some("svga_s3".into()),
            memsize: Some(32),
            vmemsize: Some("4".into()),
            xms: Some(true),
            ems: Some("emm386".into()),
            umb: Some(false),
            core: Some("dynamic".into()),
            cputype: Some("pentium_slow".into()),
            cycles: Some("fixed 3000".into()),
            aspect: Some(true),
            glshader: Some("crt-auto".into()),
            sbtype: Some("sb16".into()),
            oplmode: Some("opl3".into()),
            sbbase: Some("240".into()),
            sbirq: Some("5".into()),
            sbdma: Some("1".into()),
            sbhdma: Some("6".into()),
            rate: Some(49716),
            gus: Some(true),
            gusbase: Some("260".into()),
            gusirq: Some("11".into()),
            gusdma: Some("3".into()),
            mididevice: Some("fluidsynth".into()),
            mpu401: Some("uart".into()),
            soundfont: Some("/sf/gm.sf2".into()),
            pcspeaker: Some("discrete".into()),
            tandy: Some("on".into()),
            keyboardlayout: Some("hu".into()),
            mouse_capture: Some("seamless".into()),
            mouse_sensitivity: Some("80".into()),
            joysticktype: Some("4axis".into()),
            joy_autofire: Some(true),
            joy_swap34: Some(false),
            dos_ver: Some("6.22".into()),
            country: Some("36".into()),
            passthrough,
        }
    }

    /// The importer must understand every key the renderer writes.
    ///
    /// Regression guard: the v0.4.0 Sound Blaster IO and Gravis UltraSound fields
    /// were rendered but never imported, so they silently fell through to
    /// passthrough and the Sound tab showed them as unset.
    #[test]
    fn every_typed_key_survives_render_then_import() {
        let original = fully_populated();
        let run = RunSpec {
            mounts: vec![
                Mount {
                    drive: 'C',
                    kind: MountKind::Directory,
                    path: "/games/dune 2".into(),
                    label: Some("DUNE".into()),
                },
                Mount {
                    drive: 'D',
                    kind: MountKind::CdImage,
                    path: "/iso/dune.cue".into(),
                    label: None,
                },
            ],
            working_drive: 'C',
            command: "DUNE2.EXE".into(),
            args: vec!["/nosound".into()],
            exit_after: true,
        };

        let imported = import_profile(&original.render(&run), "Dune II");

        assert_eq!(
            imported.dosbox, original,
            "a rendered key was not mapped back by conf_import::keys"
        );
        assert_eq!(imported.run, run, "the [autoexec] block did not round-trip");
    }
}
