# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Sound tab now also covers: **OPL mode** (FM synth model) for Sound Blaster,
  **MPU-401** mode and a **SoundFont** picker for MIDI (General MIDI via
  FluidSynth), and a **PC speaker / Tandy** group — first-class fields instead
  of the Advanced passthrough.
- New **Memory** tab: RAM size (moved here from Graphics), **Video memory**
  (`vmemsize`), and the DOS memory managers **XMS**, **EMS** (true/emsboard/
  emm386/false), and **UMB** — common knobs for old games that need EMS/XMS off.
- Graphics tab: **Fullscreen** and **VSync** (auto/on/adaptive/off) options, and a
  **Shader** field (`[render] glshader`) with common presets (sharp, crt-auto, …)
  that also accepts any shader name.
- Sound tab: dedicated **Sound Blaster** (type, port, IRQ, DMA, high DMA) and
  **Gravis UltraSound** (enable, port, IRQ, DMA) settings, grouped into sections
  with the real default values — no more hand-editing the Advanced passthrough
  for these. The Sound Blaster model list is also complete (gb/sb1/sb2/sbpro1/
  sbpro2/sb16/ess/none).

### Changed
- Tidier Settings ▸ Application: grouped sections (DOSBox, Desktop shortcuts)
  each with a heading, description, and framed content; the Add/Remove shortcut
  buttons are stacked and equal-width.
- DOSBox settings: Cycles, Memory, and Mixer-rate are now free-text fields (with
  a presets dropdown), so you can type any value — e.g. a custom cycles count —
  instead of only picking from a list.
- "Default" now shows the DOSBox built-in value in the global Settings (e.g.
  `(default) · svga_s3`, Cycles placeholder `auto`), so it's clear what leaving a
  field unset actually does.

### Removed
- Graphics **Scaler** field: dosbox-staging no longer reads `[render] scaler`
  (it's obsolete), so the field did nothing. Use the new **Shader** (`glshader`)
  field instead. Existing `scaler` values are dropped on import and ignored when
  loading old profiles.

## [0.3.1] - 2026-06-30

### Changed
- New application icon (DOS prompt + floppy + Tux), installed at all hicolor
  sizes (16–512) for the menu, desktop, and window/taskbar — replaces the
  placeholder. Regenerate from `assets/app_icon.png` via `packaging/gen-icons.sh`.

### Fixed
- Desktop integration now refreshes the desktop-entry and icon-theme caches
  after adding/removing shortcuts, so the icon shows in the menu and on the
  desktop right away instead of after a re-login (a stale `icon-theme.cache`
  was hiding the freshly written icon).

## [0.3.0] - 2026-06-30

### Added
- Desktop integration (AppImage only): on first launch dosui *asks* whether to
  add shortcuts; on confirmation it installs an applications-menu entry and a
  launcher on your desktop (executable, marked trusted) plus the icon. The
  answer is remembered so it never nags. **Settings ▸ Application** has *Add* and
  *Remove* buttons to manage the shortcuts anytime. Skipped for installed/dev runs.
- Command-line interface: `dosui --help`, `--version`, `--install`, and
  `--uninstall` (the last two add/remove the menu + desktop shortcuts headlessly).

### Changed
- Bundle dosbox-staging 0.82.2 (was 0.82.0) in the AppImage.
- Update Rust dependencies (toml 1, thiserror 2, zip 8, which 8, directories 6)
  and GitHub Actions to their latest major versions.

## [0.2.0] - 2026-06-29

### Added
- D-Fend-style main window: category tree, a sortable details list with
  click-to-sort columns, a switchable icon view, and a tabbed preview pane
  (Screenshots / Notes / Data folder).
- Built-in "DOS Console" profile — a bare DOSBox prompt with a ready C: drive,
  re-addable from the toolbar with its own terminal icon.
- Project documentation and community health files (README, CONTRIBUTING,
  CODE_OF_CONDUCT, SECURITY, this changelog), CI, issue/PR templates, and an
  AppStream metainfo file.
- `Makefile` with `build` / `run` / `check` / `install` / `uninstall` /
  `appimage` targets (honours `PREFIX` and `DESTDIR`).
- Tag-driven AppImage release pipeline that publishes the AppImage and
  `SHA256SUMS`; the AppImage build script now auto-downloads a pinned
  dosbox-staging when no local build is present. See `docs/RELEASING.md`.
- Multi-distro build-from-source instructions (Debian/Ubuntu/Mint, Fedora, Arch,
  openSUSE).

### Changed
- README rewritten in English with install/usage/packaging sections.

## [0.1.0] - 2026-06-28

Initial working frontend.

### Added
- Profile library with cover grid and detail pane; launch via Play /
  double-click / Enter; "last played" tracking.
- Tabbed profile editor (General / Mounts & Run / CPU / Graphics / Sound / MIDI /
  Advanced) with a live `dosbox.conf` preview.
- New-profile wizard (folder → executable scan → metadata).
- Global defaults with per-profile inheritance; settings dialog.
- Menu bar, toolbar, right-click context menu, keyboard shortcuts.
- Category sidebar (genre / developer / year / favourites) and find-as-you-type
  search; favourites, duplicate, delete, open folder, bulk metadata editing.
- Import from `dosbox.conf` (D-Fend / DBGL) and zipped games (drag & drop).
- Single AppImage bundling dosbox-staging while following the host GTK theme.

[Unreleased]: https://github.com/lexandro/dosui/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/lexandro/dosui/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/lexandro/dosui/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/lexandro/dosui/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/lexandro/dosui/releases/tag/v0.1.0
