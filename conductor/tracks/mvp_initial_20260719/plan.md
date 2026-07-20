# Implementation Plan: MVP Initial Implementation

## Phase 1: Database Setup & Local Configuration
- [x] Task: Create initial database schema migrations for `users` and `credentials` tables via SQLx. (commit: 41dafdc)
- [ ] Task: Configure local PostgreSQL connection for the Axum backend.
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## Phase 2: Backend WebAuthn Logic
- [ ] Task: Integrate `webauthn-rs` into the backend.
- [ ] Task: Write tests for Registration API endpoints (challenge and register).
- [ ] Task: Implement Registration API endpoints.
- [ ] Task: Write tests for Authentication API endpoints (challenge and authenticate).
- [ ] Task: Implement Authentication API endpoints.
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## Phase 3: Frontend Testing Dashboard
- [ ] Task: Scaffold a unified WebAuthn testing dashboard using Shadcn UI.
- [ ] Task: Write tests for the Registration flow integration (`navigator.credentials.create`).
- [ ] Task: Implement the Registration flow UI and API integration.
- [ ] Task: Write tests for the Authentication flow integration (`navigator.credentials.get`).
- [ ] Task: Implement the Authentication flow UI and API integration.
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## Phase 4: Containerization & Minikube Deployment
- [ ] Task: Create containerization definitions (`Containerfile`) for frontend and backend for `buildah`.
- [ ] Task: Create Kubernetes deployment manifests in `frontend/k8s/` and `backend/k8s/` (including CloudNativePG single-node instance).
- [ ] Task: Create `scripts/deploy.sh` for local Minikube orchestration (build, load, apply).
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)
