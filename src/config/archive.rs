//! Import a zipped game into a new profile.
//!
//! Extracts the archive into a fresh profile's `game/` directory, auto-detects
//! the program to run, and writes the profile. GTK-free; the extraction half is
//! unit-tested.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::dosbox_conf::DosboxConfig;
use super::profile::{self, Mount, MountKind, Profile, RunSpec};

/// Create a profile from a zip archive: extract into `<profile>/game`, mount it
/// as C:, and run the first executable found. Returns the new profile directory.
pub fn import_archive(archive: &Path) -> Result<PathBuf> {
    let title = archive
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Imported game")
        .to_string();

    let (id, dir) = profile::new_profile_dir(&title)?;
    let game = dir.join("game");
    fs::create_dir_all(&game).with_context(|| format!("creating {}", game.display()))?;
    extract_zip(archive, &game)?;

    let command = profile::scan_executables(&game)
        .into_iter()
        .next()
        .unwrap_or_default();
    let profile = Profile {
        id,
        title,
        genre: None,
        year: None,
        developer: None,
        publisher: None,
        www: None,
        notes: None,
        cover: None,
        favorite: false,
        last_played: None,
        run: RunSpec {
            mounts: vec![Mount {
                drive: 'C',
                kind: MountKind::Directory,
                path: game,
                label: None,
            }],
            working_drive: 'C',
            command,
            args: Vec::new(),
            exit_after: true,
        },
        dosbox: DosboxConfig::default(),
    };
    profile.save(&dir)?;
    Ok(dir)
}

/// Extract a zip archive into `dest` (zip-slip safe via `enclosed_name`).
pub fn extract_zip(archive: &Path, dest: &Path) -> Result<()> {
    let file = fs::File::open(archive).with_context(|| format!("opening {}", archive.display()))?;
    let mut zip = zip::ZipArchive::new(file).with_context(|| "reading zip archive")?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let Some(rel) = entry.enclosed_name() else {
            continue; // skip unsafe paths (e.g. "../")
        };
        let out = dest.join(rel);
        if entry.is_dir() {
            fs::create_dir_all(&out)?;
        } else {
            if let Some(parent) = out.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut sink = fs::File::create(&out)?;
            io::copy(&mut entry, &mut sink)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn extract_zip_writes_files() {
        let tmp = tempfile::tempdir().unwrap();
        let zip_path = tmp.path().join("game.zip");

        // Build a tiny zip with one nested file.
        let mut zw = zip::ZipWriter::new(fs::File::create(&zip_path).unwrap());
        let opts: zip::write::FileOptions<()> = zip::write::FileOptions::default();
        zw.start_file("GAME.EXE", opts).unwrap();
        zw.write_all(b"MZ").unwrap();
        zw.start_file("data/readme.txt", opts).unwrap();
        zw.write_all(b"hi").unwrap();
        zw.finish().unwrap();

        let dest = tmp.path().join("out");
        extract_zip(&zip_path, &dest).unwrap();
        assert!(dest.join("GAME.EXE").is_file());
        assert!(dest.join("data/readme.txt").is_file());
    }
}
