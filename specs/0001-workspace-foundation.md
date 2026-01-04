# Workspace Foundation

## Scope
Set up the Rust workspace and baseline project layout.

## Tasks
- Create a Cargo workspace with `canvas-cli`, `canvas-core`, `canvas-models`.
- Configure shared Rust edition and lint settings.
- Set up `clap` and logging dependencies in `canvas-cli`.
- Define a consistent error type in `canvas-core`.

## Acceptance Criteria
- `cargo build` succeeds.
- `canvas --help` runs and shows top-level commands.
- Workspace compiles without warnings.

## Notes
- Keep binaries thin; push logic into `canvas-core`.
