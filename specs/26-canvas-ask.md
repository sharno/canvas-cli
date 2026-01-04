# Canvas Ask (Natural Language)

## Scope
Natural-language entrypoint for non-technical users and LLM agents.

## Tasks
- Implement `canvas ask "<prompt>"` as a first-class command.
- Convert prompt to a structured command plan (no execution by default).
- Require explicit confirmation (`--confirm`) before executing planned actions.
- Emit machine-readable JSON plan with stable schema.
- Support `--explain` to show rationale and mapped commands.

## Acceptance Criteria
- `canvas ask` returns a plan without side effects.
- `--confirm` is required to apply changes.
- JSON plan includes command IDs, parameters, and risks.

## Notes
- Keep model/provider pluggable to avoid lock-in.
