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

## JSON Output
- Every `--json` response includes stable top-level keys: `ok`, `schema_version`, and `command`.
- Success responses carry `data` with command-specific keys; error responses include `error` with `code`, `message`, and `details`.
- Use `canvas --schema` to retrieve the JSON schemas for each command group and confirm the current `schema_version`.

## Configuration
- Config file path: `~/.config/canvas-cli/config.toml`
- Environment overrides file values: `CANVAS_HOST` and `CANVAS_TOKEN` win over `auth.host` and `auth.token`.

## Milestones
1) MVP: auth, course list/select, assignment list, submission grade.
2) Expand: assignment CRUD, file upload, pages/modules.
3) Communication: announcements, messages, discussions.
4) Analytics and reports.
5) LLM schema output stabilization.
