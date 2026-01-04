# Gradebook Policies

## Scope
Gradebook posting and grading periods.

## Tasks
- Implement grading periods list and show.
- Implement posting policy get/set.
- Implement grade change log export (if available).

## Acceptance Criteria
- Policy updates require `--confirm`.
- JSON output includes policy state and affected scope.

## Notes
- Some endpoints are account-level; restrict to course scope.
