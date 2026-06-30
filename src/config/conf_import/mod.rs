//! Import an existing `dosbox.conf` into a dosui [`Profile`].
//!
//! This is what "import from D-Fend / DBGL" means in practice: both ultimately
//! produce a `dosbox.conf`. We parse the `[autoexec]` block into mounts + a run
//! command ([`autoexec`]), map the keys we model into a typed [`DosboxConfig`]
//! and keep the rest as passthrough ([`keys`]). Best-effort and GTK-free.

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
        if trimmed.is_empty() || trimmed.starts_with('#') {
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
    use crate::config::profile::MountKind;

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
}
