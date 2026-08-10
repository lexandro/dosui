# Contributing to dosui

Thanks for your interest in improving dosui! This guide covers the dev workflow,
the project's coding principles, and how the codebase is laid out.

By participating you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

## Getting started

Prerequisites (Debian / Ubuntu / Linux Mint):

```sh
sudo apt install build-essential libgtk-4-dev librsvg2-common
```

A stable Rust toolchain (MSRV **1.88**) is required. Common commands:

```sh
cargo run                  # run (RUST_LOG=debug cargo run for verbose logs)
cargo test                 # unit tests — the GTK-free core (config/, launcher logic)
cargo clippy --all-targets # lint; keep it warning-free
cargo fmt                  # format
```

Run `cargo fmt` and `cargo clippy` before every commit — CI enforces both.

## Project layout

dosui keeps a hard boundary between a **GTK-free core** and the **UI**:

- `src/config/` — pure, unit-testable Rust: profiles, settings, defaults, XDG
  paths, and `dosbox.conf` generation/inheritance. **No `gtk::` types here.**
- `src/launcher.rs` — spawns DOSBox (`gio::Subprocess`, non-blocking, `-conf`).
- `src/ui/` — GTK 4 widgets only. Calls *into* `config/`, never the reverse.

A deeper map lives in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). The core is
testable without a display server, which is your safety net — add `#[test]`s in
`config/` for any logic you change there.

## Coding principles

SOLID / DRY / KISS / YAGNI as **defaults you can break with a reason**:

- **KISS / YAGNI first.** Build what the current milestone needs; delete dead
  code immediately. No speculative config knobs.
- **DRY within reason.** Extract on the *third* repeat, not the first.
- **One concept per file; small files** (≤150 lines is a soft cap — going over is
  fine when justified, e.g. a cohesive data model plus its inline tests or a
  single GTK widget tree; note the justification in the module doc comment).
- **Explicit over clever.** Clear names, concrete types, obvious control flow.
- Errors: `anyhow` at app/UI edges, `thiserror` for typed core errors. No
  `unwrap()` on fallible paths in non-test code — surface a `Result` and a
  user-facing message.

> Note: dosui is written to be maintained primarily by AI tooling — see
> [`CLAUDE.md`](CLAUDE.md) for the full set of conventions. Human contributions
> are very welcome and follow the same principles.

## Commits & pull requests

- Use scoped, conventional messages: `feat: …`, `fix: …`, `refactor: …`,
  `docs: …`, `chore: …`.
- Keep PRs focused; one logical change per PR.
- Make sure `cargo fmt --check`, `cargo clippy`, and `cargo test` pass locally —
  CI runs all three.
- Update [CHANGELOG.md](CHANGELOG.md) under the `Unreleased` section for any
  user-visible change.
- Open an issue first for larger features so we can agree on the approach.

## Reporting bugs & requesting features

Use the issue templates. For bugs, include your distro, GTK version
(`pkg-config --modversion gtk4`), how you installed dosui (AppImage vs. source),
and steps to reproduce. For security issues, see [SECURITY.md](SECURITY.md).
