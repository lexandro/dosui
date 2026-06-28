//! Launching the DOSBox engine.
//!
//! Implemented in M1: resolve the dosbox binary (settings path → bundled →
//! `which`), generate the per-profile `dosbox.conf`, then spawn it
//! non-blockingly via `gio::Subprocess` with `-conf`.

// Intentionally empty for M0; see plan §2.4.
