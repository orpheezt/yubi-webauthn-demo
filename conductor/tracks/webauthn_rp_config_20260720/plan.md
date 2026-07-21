# Plan: Fix WebAuthn RP Config for K8s Deployment

## Phase 1: Configuration Fix

- [ ] Task: Update WebAuthn RP ConfigMap values
  - [ ] Change `RP_ID` from `"localhost"` to `"yubi.local"` in `k8s/backend.yaml`
  - [ ] Change `RP_ORIGIN` from `"http://localhost:3000"` to `"http://yubi.local"` in `k8s/backend.yaml`
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)
