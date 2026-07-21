# Specification: Cilium Gateway API Integration

## Overview
Enable Gateway API support in Cilium CNI on the Kubernetes cluster. Automate CRD installation, ConfigMap patching (`cilium-config` setting `enable-gateway-api: "true"`), workload restarts, and lifecycle integration into Makefile and lifecycle scripts.

## Requirements
1. Apply official Kubernetes Gateway API v1.1.0 CRDs.
2. Patch `cilium-config` ConfigMap in `kube-system` to ensure `enable-gateway-api: "true"`.
3. Restart Cilium DaemonSets (`cilium`, `cilium-envoy`) and Operator Deployment (`cilium-operator`).
4. Integrate with `scripts/setup.sh`, `scripts/verify.sh`, `scripts/teardown.sh`, and `Makefile`.
5. Validate via `make up`, `make verify`, and `make down`.

## Acceptance Criteria
- `make up` successfully applies Gateway API CRDs, patches `cilium-config`, restarts Cilium workloads, and deploys `k8s/gateway.yaml`.
- `make verify` checks and passes Gateway API readiness (`GatewayClass` and `Gateway` ready).
- `make down` cleanly removes application and Gateway API resources.
