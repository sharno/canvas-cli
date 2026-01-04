# Canvas LMS CLI (Rust) Plan

## Goals
- Provide a professor-friendly CLI for all common Canvas tasks (grading, content, communication, settings).
- Offer a stable, machine-friendly interface for LLMs (structured output, strict schemas).
- Make invalid states unrepresentable via Rust ADTs and strict typing.
- Parse and validate inputs at the edge, pass only valid ADTs internally.
- Support both interactive and non-interactive automation.

## Non-Goals (for v1)
- Student-focused features beyond what instructors need.
- Full admin (institution-wide) Canvas management.
- GUI or TUI (CLI only).

## Target Users
- Professors/instructors managing courses.
- TAs with grading/communication workflows.
- LLM agents that need deterministic commands and structured outputs.

## Key Principles
- "Parse at the edges": request/response parsing in a single layer.
- Strong types for IDs, dates, grades, rubric selections, file paths.
- Safe defaults; destructive operations require explicit confirmation flags.
- Idempotent operations where possible.
- Predictable output formats (human + JSON).

## Functional Scope

### Course Management
- List courses, select active course, show course info.
- Update course settings (start/end dates, visibility, grading schemes).
- Manage sections (list/add/edit).

### Assignments and Grading
- List assignments, create/update/delete.
- View submissions, grade, add feedback, attach files.
- Bulk grade update with CSV/JSON import.
- Rubric-based grading support.
- Late policy tools and regrade workflows.

### Content
- Pages: create/update/publish.
- Modules: create/update, reorder, publish.
- Files: upload/list/delete, folder management.
- Announcements and discussions.

### People and Communication
- List users (students, TAs).
- Send messages/announcements.
- Manage groups if enabled.

### Analytics and Reports
- Gradebook export.
- Submission status summaries.
- Late/missing assignments report.

## CLI UX Design
- Primary binary: `canvas`
- Command model: `canvas <resource> <action> [options]`
- Global options: `--course <id>`, `--json`, `--quiet`, `--confirm`, `--explain`
- Output modes:
  - Human readable (default)
  - Structured JSON (`--json`) with stable schemas for LLMs

### Example Commands
- `canvas course list`
- `canvas course set --course 123`
- `canvas assignment list --course 123`
- `canvas assignment create --course 123 --title "HW1" --points 50`
- `canvas submission grade --course 123 --assignment 456 --user 789 --score 48 --comment "Good work"`
- `canvas file upload --course 123 --path ./syllabus.pdf --folder "Syllabus"`
- `canvas init`
- `canvas ask "grade HW1 for section A"`

## Architecture Overview (Rust)

### Crates
- `canvas-cli` (binary)
- `canvas-core` (domain types and API client)
- `canvas-models` (ADT types, parsing/validation)

### Modules (core)
- `auth` (token, OAuth, config)
- `client` (HTTP, retries, rate limits)
- `resources` (course, assignment, submission, page, module, file, user)
- `io` (CLI parsing, output formatting)
- `errors` (typed errors)

## Data Modeling (ADTs)
- `CourseId`, `AssignmentId`, `UserId`, `SubmissionId` (newtypes)
- `Score` (bounded 0..max)
- `DueDate` (validated chrono)
- `FilePath` (validated exists at edge)
- `RubricSelection` (enum with validation)
- `PublishState` (enum)

## Edge Parsing and Validation
- Use `clap` for argument parsing.
- Convert raw args into typed request structs.
- Validate IDs, dates, file paths, and score ranges before calling core logic.

## API Integration
- Canvas REST API v1.
- Use token-based auth (ENV: `CANVAS_TOKEN`, `CANVAS_HOST`).
- Rate limiting with retry and backoff.
- Pagination helpers.

## LLM-Friendly Design
- `--json` for structured output using stable schemas.
- Use `--schema` to print JSON schema for each command.
- Deterministic errors with error codes.
- Avoid ambiguous prompts (provide `--confirm`).
- Add a guided setup flow (`canvas init`) for non-technical users.
- Add a natural-language entrypoint (`canvas ask`) that maps intent to commands with confirmation.
- Add `--explain` to show planned actions without executing.

## Storage and Config
- Config file: `~/.config/canvas-cli/config.toml`
- Store default course, preferred output format.
- Local cache for course/assignment lists (optional).

## Security and Safety
- Never log tokens.
- Confirm destructive actions unless `--confirm` provided.
- Support read-only mode to prevent mutations.

## Testing Strategy
- Unit tests for ADTs and validation.
- Integration tests with mocked Canvas API.
- Golden tests for JSON output schemas.

## Milestones
1) MVP: auth, course list/select, assignment list, submission grade.
2) Expand: assignment CRUD, file upload, pages/modules.
3) Communication: announcements, messages, discussions.
4) Analytics and reports.
5) LLM schema output stabilization.

## Open Questions
- Which Canvas features are highest priority beyond grading?
- Should we support OAuth flow or only API tokens for v1?
- Preferred default output: human or JSON?
