# Plan: Fix WebAuthn RP Config for K8s Deployment

## Phase 1: Configuration Fix [checkpoint: 0b62561]

- [x] Task: Update WebAuthn RP ConfigMap values 0b62561
  - [x] Change `RP_ID` from `"localhost"` to `"yubi.local"` in `k8s/backend.yaml`
  - [x] Change `RP_ORIGIN` from `"http://localhost:3000"` to `"https://yubi.local"` in `k8s/backend.yaml`
- [x] Task: Phase Verification & Checkpoint (Refer to workflow.md) 0b62561

