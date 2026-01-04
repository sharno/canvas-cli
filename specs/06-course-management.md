# Course Management

## Scope
Core course actions for instructors.

## Tasks
- Implement `canvas course list`.
- Implement `canvas course show --course <id>`.
- Implement `canvas course set --course <id>` for default course.
- Implement course settings update (dates, visibility, grading scheme).

## Acceptance Criteria
- `course list` supports `--json`.
- `course set` persists to config.
- Updates are validated before API calls.

## Notes
- Course updates should require `--confirm`.
