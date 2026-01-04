# Content Management

## Scope
Pages, modules, files, announcements, discussions.

## Tasks
- Pages: list/create/update/publish.
- Modules: list/create/update/reorder/publish.
- Files: list/upload/delete, folder management.
- Announcements and discussions: list/create.

## Acceptance Criteria
- File uploads validate path and size at edge.
- Publish states are validated enums.
- JSON output includes canonical URLs.

## Notes
- Avoid ambiguous "edit" commands; prefer `update`.
