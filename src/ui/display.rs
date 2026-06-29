//! Pure per-profile display helpers shared by the games views (icon grid and
//! details list) and the preview pane: cover resolution, the favourite-starred
//! title, the cover/console-icon paintable, and the "last played" cell text.
//!
//! No widgets of its own — just formatting + a `Picture` filler — so both views
//! render covers and titles identically.

use std::path::{Path, PathBuf};

use gtk::prelude::*;
use gtk::{IconPaintable, Picture};

use crate::config::console;
use crate::config::profile::{self, Profile};

/// Resolve a profile's cover to an absolute path (relative covers join `dir`).
pub(crate) fn cover_path(dir: &Path, profile: &Profile) -> Option<PathBuf> {
    profile.cover.as_ref().map(|c| {
        if c.is_absolute() {
            c.clone()
        } else {
            dir.join(c)
        }
    })
}

/// Title with a leading star for favorites.
pub(crate) fn display_title(profile: &Profile) -> String {
    if profile.favorite {
        format!("★ {}", profile.title)
    } else {
        profile.title.clone()
    }
}

/// "Last played" cell text: a coarse "time ago", or "Never".
pub(crate) fn last_played_cell(profile: &Profile) -> String {
    match profile.last_played {
        Some(then) => profile::humanize_since(profile::now_unix(), then),
        None => "Never".to_string(),
    }
}

/// Fill a `Picture` with the profile's cover image, falling back to a terminal
/// icon for console profiles. Returns whether anything was shown — callers that
/// want to collapse an empty slot use this; fixed-slot callers ignore it.
pub(crate) fn apply_cover(cover: &Picture, dir: &Path, profile: &Profile) -> bool {
    match cover_path(dir, profile) {
        Some(p) if p.exists() => {
            cover.set_filename(p.to_str());
            true
        }
        _ if console::is_console(profile) => {
            cover.set_paintable(Some(&console_paintable(cover)));
            true
        }
        _ => {
            cover.set_filename(None::<&str>);
            false
        }
    }
}

/// A themed terminal icon to stand in as the DOS console's cover.
pub(crate) fn console_paintable(widget: &impl IsA<gtk::Widget>) -> IconPaintable {
    gtk::IconTheme::for_display(&widget.display()).lookup_icon(
        "utilities-terminal",
        &[],
        96,
        1,
        gtk::TextDirection::None,
        gtk::IconLookupFlags::empty(),
    )
}
