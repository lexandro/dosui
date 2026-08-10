//! Parse the `[autoexec]` block into a [`RunSpec`]: mounts, working drive, the
//! run command + args, and whether DOSBox should exit afterwards.

use crate::config::profile::{Mount, MountKind, RunSpec};

/// Parse the `[autoexec]` lines into a [`RunSpec`].
pub(super) fn parse_run(autoexec: &[String]) -> RunSpec {
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
    let mut label = None;
    let mut i = 2;
    while i < tokens.len() {
        match tokens[i].as_str() {
            "-t" => {
                mtype = tokens.get(i + 1).cloned();
                i += 2;
            }
            "-label" => {
                label = tokens.get(i + 1).cloned();
                i += 2;
            }
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
        label,
    })
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
