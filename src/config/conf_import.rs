//! Import an existing `dosbox.conf` into a dosui [`Profile`].
//!
//! This is what "import from D-Fend / DBGL" means in practice: both ultimately
//! produce a `dosbox.conf`. We parse the `[autoexec]` block into mounts + a run
//! command, map the keys we model into a typed [`DosboxConfig`], and keep the
//! rest as passthrough. Best-effort and fully GTK-free, so it is unit-testable.

use indexmap::IndexMap;

use super::dosbox_conf::DosboxConfig;
use super::profile::{Mount, MountKind, Profile, RunSpec};

/// Build a profile from `dosbox.conf` text. `title` seeds the display name/id.
pub fn import_profile(text: &str, title: &str) -> Profile {
    let (sections, autoexec) = parse_ini(text);
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
        run: parse_run(&autoexec),
        dosbox: parse_dosbox(sections),
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

/// Parse the `[autoexec]` lines into a [`RunSpec`].
fn parse_run(autoexec: &[String]) -> RunSpec {
    let mut mounts = Vec::new();
    let mut working_drive = 'C';
    let mut command = String::new();
    let mut args = Vec::new();
    let mut exit_after = false;

    for line in autoexec {
        let l = line.trim();
        let low = l.to_lowercase();
        if l.is_empty() || l.starts_with('#') || low.starts_with("rem ") {
            continue;
        }
        if low.starts_with("mount ") || low.starts_with("imgmount ") {
            if let Some(m) = parse_mount(&tokenize(l)) {
                mounts.push(m);
            }
        } else if is_drive_switch(l) {
            working_drive = l.chars().next().unwrap().to_ascii_uppercase();
        } else if low == "exit" {
            exit_after = true;
        } else if command.is_empty() {
            let tokens = tokenize(l);
            if let Some((cmd, rest)) = tokens.split_first() {
                command = cmd.clone();
                args = rest.to_vec();
            }
        }
    }

    RunSpec {
        mounts,
        working_drive,
        command,
        args,
        exit_after,
    }
}

/// `C:` style drive switch?
fn is_drive_switch(line: &str) -> bool {
    let b = line.as_bytes();
    b.len() == 2 && b[0].is_ascii_alphabetic() && b[1] == b':'
}

/// One `mount`/`imgmount` line -> [`Mount`].
fn parse_mount(tokens: &[String]) -> Option<Mount> {
    let cmd = tokens.first()?.to_lowercase();
    let drive = tokens.get(1)?.chars().next()?.to_ascii_uppercase();

    let mut path = None;
    let mut mtype = None;
    let mut i = 2;
    while i < tokens.len() {
        match tokens[i].as_str() {
            "-t" => {
                mtype = tokens.get(i + 1).cloned();
                i += 2;
            }
            "-label" => i += 2, // skip the label value
            t if t.starts_with('-') => i += 1,
            t => {
                if path.is_none() {
                    path = Some(t.to_string());
                }
                i += 1;
            }
        }
    }

    let kind = if cmd == "mount" {
        MountKind::Directory
    } else {
        match mtype.as_deref() {
            Some("cdrom") => MountKind::CdImage,
            Some("floppy") => MountKind::FloppyImage,
            _ => MountKind::HddImage,
        }
    };
    Some(Mount {
        drive,
        kind,
        path: path?.into(),
        label: None,
    })
}

/// Map known keys to typed fields; everything else becomes passthrough.
fn parse_dosbox(mut sections: IndexMap<String, IndexMap<String, String>>) -> DosboxConfig {
    let mut cfg = DosboxConfig::default();

    // Pull a known key out of its section (so it doesn't also land in passthrough).
    let mut take = |section: &str, key: &str| -> Option<String> {
        sections.get_mut(section).and_then(|m| m.shift_remove(key))
    };

    cfg.output = take("sdl", "output");
    cfg.machine = take("dosbox", "machine");
    cfg.memsize = take("dosbox", "memsize").and_then(|v| v.parse().ok());
    cfg.core = take("cpu", "core");
    cfg.cputype = take("cpu", "cputype");
    cfg.cycles = take("cpu", "cycles");
    cfg.aspect = take("render", "aspect").map(|v| parse_bool(&v));
    cfg.scaler = take("render", "scaler");
    cfg.sbtype = take("sblaster", "sbtype");
    cfg.rate = take("mixer", "rate").and_then(|v| v.parse().ok());
    cfg.mididevice = take("midi", "mididevice");

    // Remaining keys -> passthrough (drop now-empty sections).
    sections.retain(|_, keys| !keys.is_empty());
    cfg.passthrough = sections;
    cfg
}

fn parse_bool(value: &str) -> bool {
    matches!(value.to_lowercase().as_str(), "true" | "on" | "1" | "yes")
}

/// Whitespace tokenizer that keeps double-quoted spans together.
fn tokenize(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    for c in line.chars() {
        match c {
            '"' => quoted = !quoted,
            c if c.is_whitespace() && !quoted => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

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
