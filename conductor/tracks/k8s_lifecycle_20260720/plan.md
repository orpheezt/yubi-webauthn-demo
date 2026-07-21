# Implementation Plan: K8s Infrastructure & Cluster Lifecycle Automation

## Phase 1: Rancher Storage Provisioner & Kubernetes Manifest Integration [checkpoint: 4044a44]
- [x] Task: Add Rancher local-path provisioner manifest and update PostgreSQL manifest 4044a44
  - [x] Add `k8s/rancher-local-path.yaml` manifest for Rancher local-path storage provisioner
  - [x] Update `k8s/postgres.yaml` to ensure compatibility with Rancher standard storage class
  - [x] Verify manifests formatting with `kubectl apply --dry-run=client`
- [x] Task: Phase Verification & Checkpoint (Refer to workflow.md) 4044a44

## Phase 2: Status & Verification Scripts Development
- [ ] Task: Create `scripts/status.sh` for cluster status reporting
  - [ ] Implement checks for StorageClasses, `local-path-storage` namespace, `cnpg-system` namespace, and `yubi` namespace resources
  - [ ] Format terminal output with section headers
- [ ] Task: Create `scripts/verify.sh` for automated cluster assertions
  - [ ] Implement assertion for default StorageClass ('standard' / Rancher)
  - [ ] Implement assertion for CNPG cluster readiness, migration job completion, and backend/frontend/gateway pod readiness
  - [ ] Set appropriate exit code (0 on success, non-zero on failure)
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## Phase 3: Setup & Teardown Lifecycle Scripts Refactoring
- [ ] Task: Refactor `scripts/setup.sh` to support full automated cluster lifecycle
  - [ ] Add Minikube cluster status check and conditional startup with storage addons disabled (`storage-provisioner=false,default-storageclass=false`)
  - [ ] Implement step-by-step setup ([1/6] to [6/6]) including Rancher storage provisioner, image building/loading, CNPG operator, and app manifests
  - [ ] Add post-deployment access info and status display
- [ ] Task: Create `scripts/teardown.sh` for `down` and `clean` operations
  - [ ] Implement `down` mode to safely delete `yubi` app namespace and resources
  - [ ] Implement `clean` mode to additionally uninstall CNPG operator, Rancher provisioner, and local build artifacts
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)

## Phase 4: Root Makefile Interface & End-to-End Testing
- [ ] Task: Create root `Makefile`
  - [ ] Define `.PHONY` targets: `help`, `up`, `status`, `verify`, `down`, `clean`
  - [ ] Delegate targets cleanly to corresponding scripts in `scripts/` with executable permissions
- [ ] Task: Phase Verification & Checkpoint (Refer to workflow.md)
