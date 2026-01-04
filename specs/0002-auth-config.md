# Auth and Config

## Scope
Token-based authentication and local config.

## Tasks
- Read `CANVAS_HOST` and `CANVAS_TOKEN` from env.
- Add config file support at `~/.config/canvas-cli/config.toml`.
- Merge config with env (env overrides file).
- Add `canvas auth check` command to validate credentials.
- Add `canvas init` guided setup to store host/token and defaults.

## Acceptance Criteria
- `canvas auth check` fails with a clear error if missing config.
- `canvas auth check` validates token against Canvas API.
- Config and env merge is deterministic and documented.
- `canvas init` writes a valid config file with required fields.

## Notes
- Do not log tokens.
