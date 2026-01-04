# Question Banks

## Scope
Question bank management for quizzes.

## Tasks
- Implement `question-bank list/create/update/delete`.
- Implement `question list/create/update/delete --bank <id>`.
- Support import/export via JSON where available.

## Acceptance Criteria
- Question types validate at edge (MCQ, essay, etc.).
- Deletions require `--confirm`.
- JSON output includes bank ID, question count, and metadata.

## Notes
- Align question schemas with Canvas Question Banks API.
