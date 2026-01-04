# API Client

## Scope
HTTP client for Canvas API v1 with retries, pagination, and errors.

## Tasks
- Build a typed HTTP client in `canvas-core`.
- Add rate limiting with retry/backoff.
- Implement pagination helpers.
- Map HTTP errors to typed errors with codes.

## Acceptance Criteria
- List endpoints handle pagination automatically.
- Retries occur only for safe idempotent requests.
- Errors are structured for human and JSON output.

## Notes
- Use a single client instance with pooled connections.
