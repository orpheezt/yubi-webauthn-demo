# Specification: K8s Infrastructure & Cluster Lifecycle Automation

## Overview
This track introduces a robust, developer-friendly infrastructure lifecycle management interface for the **Yubi WebAuthn** demonstration project. Drawing inspiration from `kubexplore`, it provides a root `Makefile` backed by modular shell scripts in `scripts/` to orchestrate cluster startup, storage provisioner setup (Rancher local-path provisioner), image builds/loading, database operator & cluster deployment, health verification, and clean teardown.

## Functional Requirements
1. **Root `Makefile` Interface**:
   - `make help`: Display available targets and descriptions.
   - `make up`: Full automated setup pipeline (Minikube check/start with storage addons disabled, Rancher storage provisioner installation, CloudNativePG operator installation, image builds/loads, K8s manifests application).
   - `make status`: Detailed real-time view of StorageClasses, CNPG database cluster, migration job, backend/frontend pods, and services.
   - `make verify`: Automated assertion check verifying Rancher StorageClass is default, CNPG database is Ready, migration job succeeded, and backend/frontend/gateway services are active.
   - `make down`: Graceful deletion of application namespace (`yubi`) and application resources.
   - `make clean`: Deep cleanup including operator removal (`cnpg-system`), Rancher provisioner removal (`local-path-storage`), and cached build artifacts.

2. **Minikube Cluster Lifecycle & Storage Provisioner**:
   - Check Minikube status upon `make up`. Prompt or start Minikube if stopped with storage addons disabled (`storage-provisioner=false,default-storageclass=false`).
   - Install Rancher local-path provisioner manifest (`k8s/rancher-local-path.yaml`) and set it as the default StorageClass (`standard`).

3. **Modular Scripts Refactoring**:
   - `scripts/setup.sh`: Numbered step-by-step setup script (`[1/6] Minikube check`, `[2/6] Rancher Local-Path Provisioner`, `[3/6] Image Builds & Loading`, `[4/6] CloudNativePG Operator`, `[5/6] App Manifests & DB Cluster`, `[6/6] Readiness Wait & Access Info`).
   - `scripts/teardown.sh`: Handles `down` and `clean` modes gracefully with `--ignore-not-found`.
   - `scripts/verify.sh`: Performs automated assertions on cluster state and prints formatted PASS/FAIL results.
   - `scripts/status.sh`: Displays formatted status outputs for StorageClasses, local-path storage, CNPG operator, and `yubi` application pods/services.

4. **Manifest & Storage Integration**:
   - Include Rancher local-path manifest in `k8s/rancher-local-path.yaml`.
   - Ensure `k8s/postgres.yaml` seamlessly utilizes the default Rancher `standard` storage class.

## Non-Functional Requirements
- Idempotent script execution (safe to run multiple times).
- Clear step indicators and execution output.
- Timeout handling on `kubectl wait` calls.

## Acceptance Criteria
- Running `make up` on a clean environment starts Minikube, provisions Rancher storage, builds & loads images, deploys CNPG and application manifests, and outputs a summary.
- Running `make status` displays clear status across namespaces.
- Running `make verify` returns exit code 0 when all assertions pass.
- Running `make down` removes the `yubi` namespace.
- Running `make clean` performs full system cleanup.

## Out of Scope
- Multi-node production cloud provisioning.
