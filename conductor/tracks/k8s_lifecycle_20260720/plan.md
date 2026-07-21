# Implementation Plan: K8s Infrastructure & Cluster Lifecycle Automation

## Phase 1: Rancher Storage Provisioner & Kubernetes Manifest Integration [checkpoint: 4044a44]
- [x] Task: Add Rancher local-path provisioner manifest and update PostgreSQL manifest 4044a44
  - [x] Add `k8s/rancher-local-path.yaml` manifest for Rancher local-path storage provisioner
  - [x] Update `k8s/postgres.yaml` to ensure compatibility with Rancher standard storage class
  - [x] Verify manifests formatting with `kubectl apply --dry-run=client`
- [x] Task: Phase Verification & Checkpoint (Refer to workflow.md) 4044a44

## Phase 2: Status & Verification Scripts Development [checkpoint: 639436d]
- [x] Task: Create `scripts/status.sh` for cluster status reporting 7f39f9a
  - [x] Implement checks for StorageClasses, `local-path-storage` namespace, `cnpg-system` namespace, and `yubi` namespace resources
  - [x] Format terminal output with section headers
- [x] Task: Create `scripts/verify.sh` for automated cluster assertions 639436d
  - [x] Implement assertion for default StorageClass ('standard' / Rancher)
  - [x] Implement assertion for CNPG cluster readiness, migration job completion, and backend/frontend/gateway pod readiness
  - [x] Set appropriate exit code (0 on success, non-zero on failure)
- [x] Task: Phase Verification & Checkpoint (Refer to workflow.md) 639436d

## Phase 3: Setup & Teardown Lifecycle Scripts Refactoring [checkpoint: cfbdd92]
- [x] Task: Refactor `scripts/setup.sh` to support full automated cluster lifecycle cfbdd92
  - [x] Add Minikube cluster status check and conditional startup with storage addons disabled (`storage-provisioner=false,default-storageclass=false`)
  - [x] Implement step-by-step setup ([1/6] to [6/6]) including Rancher storage provisioner, image building/loading, CNPG operator, and app manifests
  - [x] Add post-deployment access info and status display
- [x] Task: Create `scripts/teardown.sh` for `down` and `clean` operations cfbdd92
  - [x] Implement `down` mode to safely delete `yubi` app namespace and resources
  - [x] Implement `clean` mode to additionally uninstall CNPG operator, Rancher provisioner, and local build artifacts
- [x] Task: Phase Verification & Checkpoint (Refer to workflow.md) cfbdd92

## Phase 4: Root Makefile Interface & End-to-End Testing [checkpoint: e9c2aaf]
- [x] Task: Create root `Makefile` e9c2aaf
  - [x] Define `.PHONY` targets: `help`, `up`, `status`, `verify`, `down`, `clean`
  - [x] Delegate targets cleanly to corresponding scripts in `scripts/` with executable permissions
- [x] Task: Phase Verification & Checkpoint (Refer to workflow.md) e9c2aaf
