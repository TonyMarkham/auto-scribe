# AGENTS.md

This file is the durable session initializer for future agents working in this
repo. Read it before making changes.

## Project Purpose

This repository is currently a small Rust workspace and research sandbox. Keep
the project direction generic until the user defines a more specific product or
architecture.

The current implementation surface is intentionally minimal:

- `Cargo.toml`: workspace manifest.
- `crates/auto-scribe`: GPUI desktop app for push-to-talk transcription.
- `submodules/gpui-component`: local reference for GPUI component patterns.
- `submodules/zed`: local reference for GPUI, Zed UI architecture, and larger
  Rust workspace patterns.

Prefer local source inspection over web search. Use the submodules as research
inputs, not as files to edit, unless the user explicitly asks to modify or
update them.

## Repository Shape

Root files:

- `AGENTS.md`: this initializer.
- `Cargo.toml`: Rust workspace configuration, shared dependencies, and lints.
- `.gitmodules`: external research repositories checked out under
  `submodules/`.

Rust workspace:

- Workspace edition is Rust 2024.
- Workspace resolver is version 3.
- Workspace lints currently deny `unwrap_used`, `expect_used`, `panic`, and
  `unused_must_use`.
- Workspace dependencies include `thiserror`, `error-location`, `gpui`, and
  `gpui-component`.
- Auto Scribe downloads Nemotron ONNX model files into the XDG app data
  directory at runtime. Do not add large model artifacts back to Git unless the
  user explicitly asks for that packaging strategy.

Submodules:

- `submodules/gpui-component`: GPUI component and example reference.
- `submodules/zed`: GPUI source and Zed architecture reference.

Use `git submodule status` to capture exact revisions when documenting durable
findings.

## Working Rules for Agents

Evidence:

- Use `rg` / `rg --files` first for repository search.
- Prefer local submodule source over web search for GPUI, Zed, or dependency
  behavior that is present in this checkout.
- Browse only for facts that are current, external, or not available locally.
- When documenting claims, include local file references where possible.

Editing:

- Keep edits focused, reversible, and consistent with nearby code.
- Do not revert user changes.
- Use `apply_patch` for manual file edits.
- Treat submodules as read-only research inputs unless the user explicitly asks
  to change or update one.
- Add new external repositories under `submodules/`.
- Avoid broad abstractions until there is concrete repeated shape in the local
  code.

Rust style:

- Prefer typed errors with `thiserror` and `error-location`.
- Do not add `anyhow` to project crates unless the user explicitly chooses that
  direction.
- Keep `unwrap`, `expect`, and `panic` out of project code; the workspace lints
  deny them.
- Use exactly one Rust type per module file in project crates. No exceptions.
- Keep module files small and name files after the primary type they define.
- Use `mod.rs` to wire modules together when a module directory is warranted.
- Do not use `super` imports in project crates.
- Do not use glob imports in project crates.
- Prefer explicit `crate::...` imports and explicit item imports.
- Prefer workspace-managed dependency versions and lints over per-crate drift.
- Do not present Rust work as complete while `cargo check` emits warnings.
  Treat unused imports, dead code, unused public DTOs, and stale scaffolding as
  cleanup blockers unless the API is intentionally retained, documented, and
  covered by tests.

Validation:

- For documentation-only changes, reread the changed file and inspect the diff.
- For Rust changes, run the most specific practical `cargo fmt`, `cargo check`,
  or `cargo test` command for the changed crate or workspace area.
- If validation is skipped or blocked, state exactly why.

## Useful Search Starting Points

```sh
git submodule status
rg --files
rg -n "TODO|FIXME|unwrap|expect|panic|anyhow|thiserror|error-location" Cargo.toml crates
rg -n "use super|::\\*|pub use .*::\\*" crates
rg -n "gpui|gpui-component|Component|Render|IntoElement" crates submodules/gpui-component submodules/zed/crates
```
