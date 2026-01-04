# Calendar Events

## Scope
Course calendar events and scheduling.

## Tasks
- Implement `calendar event list/create/update/delete`.
- Support course-level and section-level events.
- Support all-day events and timezone-aware timestamps.

## Acceptance Criteria
- Date/time parsing is strict and timezone-aware.
- Deletions require `--confirm`.
- JSON output includes event ID, start/end, and context type.

## Notes
- Prefer ISO-8601 input with explicit offset.
