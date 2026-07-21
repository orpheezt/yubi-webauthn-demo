# Spec: Fix WebAuthn RP Config for K8s Deployment

## Overview

The WebAuthn Relying Party configuration in the Kubernetes backend ConfigMap (`k8s/backend.yaml`) is hardcoded to `localhost` development defaults. When the application is accessed through the Cilium Gateway (hostname `yubi.local`), the browser sends the actual origin in the WebAuthn assertion, which doesn't match the configured `RP_ORIGIN`. Additionally, if the user accesses via IP (`127.0.0.1`), the `webauthn-rs` library rejects it outright because IP addresses are not valid WebAuthn RP domains.

## Root Cause

- `RP_ID` is set to `"localhost"` instead of `"yubi.local"`
- `RP_ORIGIN` is set to `"http://localhost:3000"` instead of `"http://yubi.local"`
- The backend Rust code correctly reads these from environment variables — no code changes required.

## Functional Requirements

1. Update `RP_ID` in the `yubi-backend-config` ConfigMap to `"yubi.local"`.
2. Update `RP_ORIGIN` in the `yubi-backend-config` ConfigMap to `"http://yubi.local"`.

## Non-Functional Requirements

- None.

## Acceptance Criteria

1. The `k8s/backend.yaml` ConfigMap contains `RP_ID: "yubi.local"` and `RP_ORIGIN: "http://yubi.local"`.
2. After redeployment, WebAuthn registration and authentication flows succeed when accessed via `http://yubi.local`.
3. The error `"127.0.0.1 is an invalid domain"` no longer appears.

## Out of Scope

- Backend Rust code changes (the code already reads `RP_ID` and `RP_ORIGIN` from env vars).
- Frontend changes.
- HTTPS/TLS configuration for the RP origin.
- Making `RP_ID`/`RP_ORIGIN` dynamically configurable beyond the ConfigMap.
