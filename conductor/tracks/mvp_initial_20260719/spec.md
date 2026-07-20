# Specification: MVP Initial Implementation

## Overview
This track focuses on the initial implementation of the Yubi WebAuthn Demo. The goal is to establish a secure, end-to-end WebAuthn (Passkeys) registration and authentication flow. This includes the Rust backend using `webauthn-rs`, a React frontend with a unified testing dashboard, and a local Minikube deployment orchestrated via bash scripts.

## Functional Requirements
- **Backend:**
  - Implement WebAuthn server-side logic using `webauthn-rs`.
  - Expose REST APIs for Registration (challenge, register) and Authentication (challenge, authenticate).
  - Manage user and credential storage in PostgreSQL using a simple schema (`users` and `credentials` tables).
- **Frontend:**
  - Create a unified dashboard using Shadcn UI for testing both Registration and Authentication flows seamlessly.
  - Integrate with the browser's `navigator.credentials` API.
- **Infrastructure:**
  - Create a `scripts/deploy.sh` script to build images, load them into Minikube (`minikube image load`), and `kubectl apply` the manifests.
  - Configure a single-node CloudNativePG instance for the PostgreSQL database in Kubernetes.
  - Provide `k8s/` manifests in both `frontend` and `backend` directories.

## Non-Functional Requirements
- **Containerization:** Both frontend and backend must be containerized using `buildah`.
- **Database:** PostgreSQL must be managed via CloudNativePG.

## Acceptance Criteria
- [ ] A user can successfully register a new account using a passkey/hardware key.
- [ ] The registered user can subsequently authenticate successfully using the same passkey.
- [ ] User and credential data are persistently stored in the PostgreSQL database.
- [ ] Running `scripts/deploy.sh` successfully sets up the entire application (frontend, backend, DB) in a local Minikube cluster and it is accessible via the browser.

## Out of Scope
- Production Ingress/TLS configuration.
- Advanced user management (e.g., password reset, account recovery).
- Fallback authentication methods (e.g., passwords).
