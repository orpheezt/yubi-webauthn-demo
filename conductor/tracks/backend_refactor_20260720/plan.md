# Implementation Plan: Backend REST API Refactor

## Phase 1: Build & Setup Optimizations
- [x] 8063e76 Task: Resolve `sqlx` query cache build failure
  - [ ] Start database services if required for sqlx preparation
  - [ ] Run `cargo sqlx prepare` to update `sqlx-data.json`
  - [ ] Verify `cargo check` and `cargo build` pass without `SQLX_OFFLINE` errors
- [x] 8063e76 Task: Apply Compiler Optimizations
  - [ ] Update `Cargo.toml` release profile to include `lto = true` and any other relevant optimizations (e.g., `codegen-units = 1`)
- [x] Task: Phase Verification & Checkpoint (Refer to workflow.md) [checkpoint: cf5c733]

## Phase 2: Refactor Database Access Layer
- [x] 4a38aa6 Task: Identify and isolate common DB logic
  - [x] Review `src/routes.rs` (or equivalent) for duplicated database queries/logic
  - [x] Create a new module (e.g., `src/repository.rs` or `src/db.rs`) if appropriate, and ensure tests cover the expected access patterns
- [x] 4a38aa6 Task: Implement shared DB components
  - [x] Extract the identified common database logic into the new shared functions
  - [x] Clean up redundant comments related to the extracted logic
- [x] Task: Phase Verification & Checkpoint (Refer to workflow.md) [checkpoint: 4a38aa6]

## Phase 3: Refactor Axum Routes
- [x] 86d76a3 Task: Update Route Handlers
  - [x] Modify existing Axum route handlers to utilize the newly extracted shared database components
  - [x] Eliminate duplicated request validation or error handling logic by utilizing Axum extractors or middleware if applicable
  - [x] Clean up redundant comments in route handlers
- [x] Task: Phase Verification & Checkpoint (Refer to workflow.md) [checkpoint: 86d76a3]

## Phase 4: Final Cleanup and Validation
- [x] a9a7f43 Task: Codebase Cleanup
  - [x] Run `cargo fmt` and `cargo clippy` to ensure code style compliance and remove any unused imports or dead code identified during refactoring
- [x] a9a7f43 Task: Test Validation
  - [x] Run full test suite (`cargo test`) to ensure all tests pass and coverage is >80%
- [x] Task: Phase Verification & Checkpoint (Refer to workflow.md) [checkpoint: a9a7f43]
