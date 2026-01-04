# Assignments and Grading

## Scope
Assignment CRUD and grading workflows.

## Tasks
- Implement `assignment list/create/update/delete`.
- Implement `submission list --assignment <id>`.
- Implement `submission grade --assignment <id> --user <id> --score <n>`.
- Add rubric-based grading inputs.
- Add bulk grade import from CSV/JSON.

## Acceptance Criteria
- Scores are validated against assignment points.
- Bulk import reports per-row success/failure.
- JSON output includes stable identifiers and status.

## Notes
- Destructive actions require `--confirm`.
