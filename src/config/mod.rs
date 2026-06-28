//! Non-UI core: XDG paths, app settings, profiles and dosbox.conf generation.
//!
//! Everything in this module is GTK-free so it can be unit-tested without a
//! display server (see the plan's verification section).

pub mod paths;

// Filled in during M1+:
// pub mod settings;
// pub mod defaults;
// pub mod profile;
// pub mod dosbox_conf;
