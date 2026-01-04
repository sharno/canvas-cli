# LLM Output Schemas

## Scope
Structured output and schema discovery.

## Tasks
- Define JSON schema per command group.
- Implement `canvas --schema` to emit schemas.
- Add error codes and consistent error JSON shapes.
- Document stable keys and versioning.

## Acceptance Criteria
- `--json` output validates against schema.
- Errors include `code`, `message`, `details`.
- Schema versions are explicit.

## Notes
- Prefer flat JSON for easy parsing.
