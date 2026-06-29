# Architecture

dosui is a **Rust + GTK 4** desktop application. Its single most important design
rule is a hard boundary between a **GTK-free core** and the **UI layer**.

```
            ┌──────────────────────────────────────────┐
            │  src/ui/        (GTK 4 widgets only)       │
            │  main_window, games_view, list_view, grid, │
            │  preview*, profile_editor, wizard, …        │
            └───────────────────────┬───────────────────┘
                                    │ calls into (never the reverse)
            ┌───────────────────────▼───────────────────┐
            │  src/config/    (pure, unit-testable Rust) │
            │  profile, settings, defaults, paths,       │
            │  dosbox_conf, conf_import, archive          │
            └───────────────────────┬───────────────────┘
                                    │
            ┌───────────────────────▼───────────────────┐
            │  src/launcher.rs   gio::Subprocess → dosbox │
            └────────────────────────────────────────────┘
```

## Layers

### `src/config/` — the core

Pure Rust with **no `gtk::` types**, so it can be unit-tested without a display
server. Responsibilities:

- **`profile.rs`** — the `Profile` data model and `RunSpec` (what to mount and
  run). Round-trips to `profile.toml`, one directory per profile.
- **`settings.rs` / `defaults.rs`** — app settings and the global `DosboxConfig`
  defaults every profile inherits from.
- **`dosbox_conf.rs`** — renders the `[autoexec]` and typed sections into a
  `dosbox.conf`, merging global defaults with per-profile overrides.
- **`paths.rs`** — XDG path resolution (config / data / profiles).
- **`conf_import.rs` / `archive.rs`** — import an existing `dosbox.conf`
  (D-Fend / DBGL) or a zipped game.

### `src/launcher.rs`

Regenerates the profile's `dosbox.conf`, resolves the DOSBox binary (explicit
setting → bundled AppImage → `PATH`), then spawns it non-blocking via
`gio::Subprocess` with `-conf`. Mounts and the run command live in `[autoexec]`,
so this is engine-agnostic across dosbox-staging / dosbox-x / vanilla.

### `src/ui/`

GTK 4 widgets only; calls *into* `config/`, never the reverse. Notable modules:

- **`main_window.rs`** — orchestration: assembles the menu bar, toolbar, sidebar,
  the games view, and the preview pane; owns the model chain and reload.
- **`games_view.rs`** — a `Stack` holding the details `ColumnView` (`list_view`)
  and the icon `GridView` (`grid`), driven by one shared selection.
- **`preview*.rs`** — the bottom tabbed pane (Screenshots / Notes / Data folder).
- **`profile_editor.rs`** + `editor_*` / `dosbox_form` — the tabbed editor.
- **`wizard*.rs`** — the new-profile flow.

## Data flow

A launch is: **`Profile` → `RunSpec` → rendered `dosbox.conf` → DOSBox**.

```
defaults.toml ─┐
               ├─ merge ─► effective DosboxConfig ─► render(run) ─► dosbox.conf ─► dosbox -conf
profile.toml ──┘
```

## Storage

- dosui's own files are **TOML**: `~/.config/dosui/` (settings, defaults) and
  `~/.local/share/dosui/profiles/` (one subdirectory per profile — the valuable
  content).
- The generated `dosbox.conf` is an **INI output artifact** — never a source of
  truth.

## Testing

Logic that can be tested without a display lives in `config/` with `#[test]`s.
That core is the safety net: `cargo test` exercises profile round-trips,
`dosbox.conf` generation, import, and path logic with no GTK dependency. The UI
is verified by running the app.

## Conventions

One concept per file, small files (≤150-line soft cap), explicit over clever,
errors surfaced as `Result`. See [CONTRIBUTING.md](../CONTRIBUTING.md) and
[CLAUDE.md](../CLAUDE.md) for the full set.
