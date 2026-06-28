# CLAUDE.md — dosui

Lightweight **native Linux frontend for DOSBox**, D-Fend Reloaded style.
Stack: **Rust + GTK4 (`gtk4-rs`, no libadwaita)**. Engine: dosbox-staging (path from
config). Target: Linux Mint / Cinnamon / X11. Ship: single AppImage with bundled dosbox.

Full design + milestones: `~/.claude/plans/mutable-weaving-starlight.md`.

## Commands
```
cargo build            # debug build
cargo run              # run (RUST_LOG=debug cargo run for verbose)
cargo test             # unit tests — run the GTK-free core (config/, launcher logic)
cargo clippy           # lint; keep it clean before committing
cargo fmt              # format; run before every commit
```

## Architecture
- `src/config/` — **GTK-free core**. Pure Rust, unit-testable without a display.
  Profiles, settings, defaults, XDG paths, and `dosbox.conf` generation/inheritance.
- `src/launcher.rs` — spawn dosbox (`gio::Subprocess`, non-blocking, `-conf`).
- `src/ui/` — GTK4 widgets only. Calls *into* `config/`; never the reverse.
- **Hard rule:** no GTK types in `config/`. This keeps the core testable and the
  UI swappable. If you reach for `gtk::` inside `config/`, the design is wrong.

Storage: dosui's own files are **TOML** (`~/.config/dosui`, `~/.local/share/dosui`).
The generated `dosbox.conf` is an **INI output artifact** — never a source of truth.

## Code principles (pragmatic, not dogmatic)
SOLID / DRY / KISS / YAGNI as **defaults you can break with a reason**:
- **KISS / YAGNI first.** Build what the current milestone needs. No speculative
  generality, no config knobs nobody asked for. Delete dead code immediately.
- **DRY within reason.** Extract on the *third* repeat, not the first. A little
  duplication beats the wrong abstraction.
- **SOLID where it pays.** Single-purpose modules/functions; depend on small traits
  only where a real second implementation exists or is imminent. Don't add traits/
  generics "for flexibility" — that's YAGNI.
- Prefer **flat over nested**, early returns, small functions.
- Errors: `anyhow` at app/UI edges, `thiserror` for typed core errors. No `unwrap()`
  in non-test code on fallible paths — surface a `Result` and a user-facing message.

## Write AI-maintained code (primary directive)
**No human will edit this code — only an AI will.** Optimize for an LLM's ability to
load, understand, and safely change it with **minimum tokens**. Concretely:
- **One concept per file; small files.** The AI should load only what it needs. If a
  file does two things, split it. Filename = concept (greppable).
- **Explicit over clever.** Clear names, concrete types, obvious control flow. No
  macro magic, hidden side effects, or deep indirection — every hop costs the AI tokens
  and risk. Code that's boring to read is cheap to change.
- **State intent at the top.** Each module/non-trivial fn gets a short doc comment with
  *purpose + invariants*. Comments explain **why**, never restate **what**.
- **Self-contained units.** Minimize cross-file coupling so a change needs few files in
  context. Stable public APIs; avoid churn that forces re-reading callers.
- **Keep it verifiable.** Logic that can be tested without a display lives in `config/`
  with `#[test]`s — the AI's safety net for self-checking a change.
- **Token economy in output too.** Prefer concise, dense, scannable code and docs over
  verbose prose. Don't pad with ceremony.
- Tension rule: when "nice for humans" conflicts with "cheap for an AI to maintain
  correctly", **choose the AI.**

## Commits
- Commit at **every milestone end**, and after **larger steps within a milestone**.
- Structured, scoped messages: `feat(M1): …`, `chore(M0): …`, `docs: …`, `fix: …`.
- Run `cargo fmt` + `cargo clippy` before committing.
