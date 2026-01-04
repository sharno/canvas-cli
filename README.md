# Canvas CLI Overview

## Goals
- Professor-friendly CLI for common Canvas tasks.
- Stable, machine-friendly interface for LLMs.
- Rust ADTs and strict typing to make invalid state unrepresentable.
- Parse and validate at the edges, internal logic uses only valid ADTs.

## Non-Goals (v1)
- Student-only features.
- Institution-wide admin features.
- GUI or TUI.

## Deliverables
- `canvas` CLI with human and JSON outputs.
- Rust workspace with `canvas-cli`, `canvas-core`, `canvas-models`.
- Core workflows: auth, course list/select, assignment list, submission grade.

## Configuration
- Config file path: `~/.config/canvas-cli/config.toml`
- Environment overrides file values: `CANVAS_HOST` and `CANVAS_TOKEN` win over `auth.host` and `auth.token`.

## Milestones
1) MVP: auth, course list/select, assignment list, submission grade.
2) Expand: assignment CRUD, file upload, pages/modules.
3) Communication: announcements, messages, discussions.
4) Analytics and reports.
5) LLM schema output stabilization.
