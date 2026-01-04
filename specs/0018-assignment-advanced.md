# Assignment Advanced Features

## Scope
Instructor features beyond basic assignment CRUD.

## Tasks
- Support assignment overrides (section/student due dates).
- Support peer reviews (automatic/manual, due dates).
- Support group assignments and group categories.
- Expose grading posting policy and muted state per assignment.

## Acceptance Criteria
- Overrides validate target type and date ranges.
- Peer review settings are validated at edge.
- JSON output includes override counts and group settings.

## Notes
- Overrides often affect downstream grade validation.
