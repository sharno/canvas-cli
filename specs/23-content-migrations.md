# Content Migrations

## Scope
Import/export and course copy workflows.

## Tasks
- Implement `content-migration list/create/show`.
- Support course copy and file import.
- Track migration status and progress.

## Acceptance Criteria
- Source and target course IDs validate at edge.
- JSON output includes migration ID and workflow state.

## Notes
- Some migrations are asynchronous; poll with backoff.
