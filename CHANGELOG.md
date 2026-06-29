# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- D-Fend-style main window: category tree, a sortable details list with
  click-to-sort columns, a switchable icon view, and a tabbed preview pane
  (Screenshots / Notes / Data folder).
- Built-in "DOS Console" profile — a bare DOSBox prompt with a ready C: drive,
  re-addable from the toolbar with its own terminal icon.
- Project documentation and community health files (README, CONTRIBUTING,
  CODE_OF_CONDUCT, SECURITY, this changelog), CI, issue/PR templates, and an
  AppStream metainfo file.

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

[Unreleased]: https://github.com/lexandro2000/dosui/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/lexandro2000/dosui/releases/tag/v0.1.0
