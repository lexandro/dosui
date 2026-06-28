//! App-wide constants and metadata.

/// Reverse-DNS application id. Used for the GTK application, GSettings, and the
/// XDG `ProjectDirs` lookup (see [`crate::config::paths`]).
pub const APP_ID: &str = "io.github.dosui";

/// Human-facing application name (window title, about dialog).
pub const APP_NAME: &str = "dosui";
