//! Profile id/directory naming: slugs, collision-free allocation, duplication.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::Profile;

/// A filesystem-safe lowercase slug for a title; "profile" when empty.
pub fn slugify(title: &str) -> String {
    let slug = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        "profile".to_string()
    } else {
        slug
    }
}

/// Pick a free profile id/dir under `root` from `title`, suffixing `-2`, `-3`…
/// on collision. Pure (takes the root) so it is unit-testable.
pub fn allocate_dir(root: &Path, title: &str) -> (String, PathBuf) {
    let base = slugify(title);
    let mut id = base.clone();
    let mut n = 2;
    while root.join(&id).exists() {
        id = format!("{base}-{n}");
        n += 1;
    }
    let dir = root.join(&id);
    (id, dir)
}

/// Allocate a new profile id/dir under the real profiles root.
pub fn new_profile_dir(title: &str) -> Result<(String, PathBuf)> {
    Ok(allocate_dir(&crate::config::paths::profiles_dir()?, title))
}

/// Duplicate a profile: create a new "<title> (copy)" profile directory next to
/// the others, copying a relative cover image. Returns the new directory.
pub fn duplicate(src_dir: &Path, profile: &Profile) -> Result<PathBuf> {
    let title = format!("{} (copy)", profile.title);
    let (id, dir) = new_profile_dir(&title)?;
    let mut copy = profile.clone();
    copy.id = id;
    copy.title = title;
    copy.last_played = None;

    fs::create_dir_all(&dir).with_context(|| format!("creating profile dir {}", dir.display()))?;
    if let Some(cover) = &copy.cover {
        if cover.is_relative() {
            let from = src_dir.join(cover);
            if from.exists() {
                let _ = fs::copy(&from, dir.join(cover));
            }
        }
    }
    copy.save(&dir)?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_normalizes_titles() {
        assert_eq!(slugify("Dune II"), "dune-ii");
        assert_eq!(slugify("  Commander Keen 4!  "), "commander-keen-4");
        assert_eq!(slugify("X-COM: UFO Defense"), "x-com-ufo-defense");
        assert_eq!(slugify(""), "profile");
        assert_eq!(slugify("***"), "profile");
    }

    #[test]
    fn allocate_dir_suffixes_on_collision() {
        let root = tempfile::tempdir().unwrap();
        let (id, dir) = allocate_dir(root.path(), "Dune II");
        assert_eq!(id, "dune-ii");
        fs::create_dir_all(&dir).unwrap();

        let (id2, _) = allocate_dir(root.path(), "Dune II");
        assert_eq!(id2, "dune-ii-2");
    }
}
