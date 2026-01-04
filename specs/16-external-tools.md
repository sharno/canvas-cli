# External Tools

## Scope
LTI external tools management.

## Tasks
- Implement `tool list/create/update/delete`.
- Support tool configuration by URL or XML/JSON.
- Support placement configuration where applicable.

## Acceptance Criteria
- Tool configs validate required fields at edge.
- Deletions require `--confirm`.
- JSON output includes tool ID, name, and placements.

## Notes
- Start with External Tools API (LTI 1.1/1.3 as supported by Canvas).
