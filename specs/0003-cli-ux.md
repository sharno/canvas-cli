# CLI UX

## Scope
Command structure, flags, output modes.

## Tasks
- Define command tree: `canvas <resource> <action>`.
- Implement global flags: `--course`, `--json`, `--quiet`, `--confirm`.
- Add `canvas --schema` for schema discovery (see LLM spec).
- Provide examples in `--help` for major commands.
- Add `--explain` to show the planned actions without executing.
- Add `canvas ask "<natural language>"` to map intent to commands.

## Acceptance Criteria
- Commands follow consistent naming and flag usage.
- `--json` outputs valid JSON with stable keys.
- `--confirm` is required for destructive actions.
- `--explain` returns the intended operations with no mutation.
- `canvas ask` produces a command plan and requires confirmation before executing.

## Notes
- Prefer short, predictable commands over nested subcommands.
