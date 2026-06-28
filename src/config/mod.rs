//! Non-UI core: XDG paths, app settings, profiles and dosbox.conf generation.
//!
//! Everything in this module is GTK-free so it can be unit-tested without a
//! display server (see the plan's verification section).

pub mod defaults;
pub mod dosbox_conf;
pub mod paths;
pub mod profile;
pub mod settings;
