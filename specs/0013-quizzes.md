# Quizzes

## Scope
Instructor quiz management and grading.

## Tasks
- Implement `quiz list/create/update/delete`.
- Implement `quiz publish/unpublish`.
- Implement `quiz submission list` and `quiz submission grade`.
- Support time limits, availability windows, and access codes.

## Acceptance Criteria
- Quiz settings validate at edge (dates, time limit, points).
- Publish actions require `--confirm`.
- JSON output includes quiz ID, due date, and published state.

## Notes
- Canvas has multiple quiz engines; start with Classic Quizzes API.
