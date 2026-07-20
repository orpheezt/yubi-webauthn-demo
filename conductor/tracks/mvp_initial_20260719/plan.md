# Implementation Plan: MVP Initial Implementation

## Phase 1: Database Setup & Local Configuration
- [x] Task: Create initial database schema migrations for `users` and `credentials` tables via SQLx. (commit: 41dafdc)
- [x] Task: Configure local PostgreSQL connection for the Axum backend. (commit: 3a6a016)
- [x] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## Phase 2: Backend WebAuthn Logic
- [x] Task: Integrate `webauthn-rs` into the backend. (commit: 8df223b)
- [x] Task: Write tests for Registration API endpoints (challenge and register). (commit: 91a7355)
- [x] Task: Implement Registration API endpoints. (commit: d9b4998)
- [x] Task: Write tests for Authentication API endpoints (challenge and authenticate). (commit: ef4b233)
- [x] Task: Implement Authentication API endpoints. (commit: 9bb8cba)
- [x] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## Phase 3: Frontend WebAuthn UI (React/TanStack)
- [x] Task: Integrate WebAuthn library (`@simplewebauthn/browser`) into the React frontend. (commit: 765962f)
- [x] Task: Scaffold a unified WebAuthn testing dashboard using Shadcn UI. (commit: 7ac8a31)
- [x] Task: Implement the Registration flow UI and wire to `/api/auth/register/*` endpoints. (commit: 7ac8a31)
- [x] Task: Implement the Authentication flow UI and wire to `/api/auth/login/*` endpoints. (commit: 7ac8a31)
- [x] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## Phase 4: Containerization & Minikube Deployment
- [ ] Task: Create containerization definitions (`Containerfile`) for frontend and backend for `buildah`.
- [ ] Task: Create Kubernetes deployment manifests in `frontend/k8s/` and `backend/k8s/` (including CloudNativePG single-node instance).
- [ ] Task: Create `scripts/deploy.sh` for local Minikube orchestration (build, load, apply).
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)
