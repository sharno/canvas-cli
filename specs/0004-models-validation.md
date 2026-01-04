# Models and Validation

## Scope
Define ADTs and edge validation.

## Tasks
- Create newtypes for IDs: `CourseId`, `AssignmentId`, `UserId`, `SubmissionId`.
- Define `Score` with bounds and validation.
- Define `DueDate` with strict parsing.
- Define `PublishState`, `RubricSelection`.
- Centralize parsing from raw CLI inputs into typed request structs.

## Acceptance Criteria
- Invalid inputs are rejected at the CLI edge.
- Core functions accept only typed request structs.
- Tests exist for model validation boundaries.

## Notes
- Use `TryFrom`/`FromStr` for parsing.
