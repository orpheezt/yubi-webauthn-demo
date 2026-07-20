# Specification: Backend REST API Refactor

## Overview
This track focuses on refactoring the backend REST API structure for the Yubi WebAuthn Demo. Specifically, it targets the Axum routes and database access layer. The primary motivation is to reduce code duplication by extracting common logic, clean up redundant code and comments, optimize the build (e.g., LTO), and resolve the current CI build failure related to `sqlx` offline query caching.

## Functional Requirements
- Identify redundant logic within Axum route handlers and database access functions.
- Extract common logic into shared utility functions, middleware, or reusable database traits/structs.
- Clean up any redundant code and outdated or unnecessary comments.
- Update existing routes and database calls to utilize the newly extracted components.
- Update the `sqlx` query cache to resolve the `SQLX_OFFLINE=true` build errors.

## Non-Functional Requirements
- Maintain code readability and adhere to Rust and Axum best practices.
- Explore and implement compiler optimizations, such as Link-Time Optimization (LTO) in the release profile, to enhance backend performance.
- The refactoring must not degrade performance (e.g., avoid introducing unnecessary allocations or synchronous blocking in async contexts).

## Acceptance Criteria
- All existing unit and integration tests continue to pass.
- Code coverage remains at or above the required >80% threshold.
- The external API contract (endpoints, request/response payloads, status codes) remains absolutely identical.
- CI pipeline completes successfully without `SQLX_OFFLINE` errors.
- Cargo release profile is updated with optimized settings (e.g., `lto = true`).

## Out of Scope
- Adding new features or API endpoints.
- Modifying frontend UI components.
- Changes to infrastructure or Kubernetes manifests (other than ensuring the build works).
