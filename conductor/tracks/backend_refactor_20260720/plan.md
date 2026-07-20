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
- [ ] Task: Identify and isolate common DB logic
  - [ ] Review `src/routes.rs` (or equivalent) for duplicated database queries/logic
  - [ ] Create a new module (e.g., `src/repository.rs` or `src/db.rs`) if appropriate, and ensure tests cover the expected access patterns
- [ ] Task: Implement shared DB components
  - [ ] Extract the identified common database logic into the new shared functions
  - [ ] Clean up redundant comments related to the extracted logic
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## Phase 3: Refactor Axum Routes
- [ ] Task: Update Route Handlers
  - [ ] Modify existing Axum route handlers to utilize the newly extracted shared database components
  - [ ] Eliminate duplicated request validation or error handling logic by utilizing Axum extractors or middleware if applicable
  - [ ] Clean up redundant comments in route handlers
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## Phase 4: Final Cleanup and Validation
- [ ] Task: Codebase Cleanup
  - [ ] Run `cargo fmt` and `cargo clippy` to ensure code style compliance and remove any unused imports or dead code identified during refactoring
- [ ] Task: Test Validation
  - [ ] Run full test suite (`cargo test`) to ensure all tests pass and coverage is >80%
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)
