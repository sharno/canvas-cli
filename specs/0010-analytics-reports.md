# Analytics and Reports

## Scope
Reporting endpoints for grading and submission status.

## Tasks
- Implement gradebook export.
- Implement submission status summary (late/missing).
- Provide course activity summaries if available.

## Acceptance Criteria
- Reports can output JSON and CSV where applicable.
- Large reports are streamed to avoid memory spikes.

## Notes
- Respect Canvas rate limits on heavy exports.
