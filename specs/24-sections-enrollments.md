# Sections and Enrollments

## Scope
Manage course sections and enrollments.

## Tasks
- Implement `section list/create/update/delete`.
- Implement `enrollment list/add/remove`.
- Support role-based enrollments and limits.

## Acceptance Criteria
- Role values validate at edge.
- Deletions require `--confirm`.
- JSON output includes section IDs and enrollment counts.

## Notes
- Enrollment changes may be restricted by institution policy.
