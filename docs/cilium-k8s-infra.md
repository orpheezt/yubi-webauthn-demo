# Cilium Kubernetes Infrastructure & Gateway API Documentation

## Overview

This project uses **Cilium** deployed via Helm (`cilium/cilium`) to manage cloud-native networking, eBPF load balancing, ingress routing via the Kubernetes Gateway API standard, and Layer-2 LoadBalancer IP allocation.

---

## Architectural Components

### 1. Cilium CNI (eBPF Datapath)
- **High-Performance eBPF Routing**: Replaces traditional Linux `iptables` rules with kernel-level eBPF TC (Traffic Control) filters for fast pod-to-pod and node-to-node networking.
- **Socket Layer Load Balancing**: Bypasses network device overhead for local pod-to-pod traffic.

### 2. Kube-Proxy Replacement (KPR)
- **Direct eBPF Service Routing**: Completely disables standard `kube-proxy`.
- **NodePort & LoadBalancer eBPF Datapath**: Handles Service IP translation directly inside eBPF hooks on host network interfaces (`eth0`).

### 3. Kubernetes Gateway API Controller
- **Gateway API Standard**: Implements `gateway.networking.k8s.io/v1` (`Gateway` and `HTTPRoute` CRDs).
- **Control Loop**: Monitors `Gateway` resources in the `yubi` namespace and translates them into `CiliumEnvoyConfig` (CEC) custom resources.

### 4. Cilium Envoy L7 Proxy
- **DaemonSet / Proxy Workload**: Containerized Envoy instance (`cilium-envoy`) running in `kube-system`.
- **Capabilities**: Handles HTTP/HTTPS routing, L7 path matching (`/api/*` -> backend, `/` -> frontend), HTTP header manipulation, and TLS termination.

### 5. Cilium LB-IPAM (LoadBalancer IP Address Management)
- **IP Address Allocation**: Uses `CiliumLoadBalancerIPPool` (`cilium.io/v2`) to dynamically assign IP addresses to Services of type `LoadBalancer`.
- **Allocated VIP**: `192.168.39.240` assigned to `cilium-gateway-yubi-gateway`.

### 6. Cilium Layer-2 Announcements
- **ARP Broadcasting**: Uses `CiliumL2AnnouncementPolicy` (`cilium.io/v2alpha1`) to advertise LoadBalancer IP addresses on local Layer-2 networks via Gratuitous ARP on `eth0`.

---

## Service Topology & Ports

| Service Name | Namespace | Service Type | IP / Port | Description |
|---|---|---|---|---|
| **`cilium-gateway-yubi-gateway`** | `yubi` | `LoadBalancer` | `192.168.39.240:80, 443` | Cilium Gateway API L7 Ingress Proxy & TLS Termination |
| **`yubi-frontend`** | `yubi` | `ClusterIP` | `10.104.193.215:3000` | TanStack Start / Nitro SSR & React SPA Frontend |
| **`yubi-backend`** | `yubi` | `ClusterIP` | `10.111.75.103:8080` | Rust Axum REST API & WebAuthn RP |
| **`yubi-pg-rw`** | `yubi` | `ClusterIP` | `10.111.195.207:5432` | CloudNativePG Primary PostgreSQL Instance |
| **`cilium-operator`** | `kube-system` | `Deployment` | Internal | Gateway API controller & Secret synchronizer |
| **`cilium-envoy`** | `kube-system` | `DaemonSet` | Host / `9999` | Cilium Envoy L7 Datapath DaemonSet |

---

## Configuration Manifests

### 1. Gateway API Resources (`k8s/gateway.yaml`)
- Defines the `yubi-gateway` `Gateway` listener on port `80` (HTTP) and `443` (HTTPS with TLS Secret `yubi-tls-secret`).
- Defines `HTTPRoute` rules:
  - Path prefix `/api` routed to `yubi-backend:8080`.
  - Path prefix `/` routed to `yubi-frontend:3000`.

### 2. Cilium LB IP Pool (`k8s/cilium-lb.yaml`)
- Configures `CiliumLoadBalancerIPPool` (`yubi-lb-pool`) with CIDR range `192.168.39.240/32`.
- Configures `CiliumL2AnnouncementPolicy` (`yubi-l2-policy`) targeting interface `eth0`.

---

## Deployment & Verification Commands

```bash
# 1. Apply Gateway API CRDs & patch Cilium configuration
make cilium-gateway

# 2. Deploy full cluster stack including Cilium Gateway
make up

# 3. Verify status of Cilium Gateway and services
make verify
```

---

## HTTPS & TLS Termination

HTTPS TLS termination on `yubi-gateway` relies on trusted certificates generated via `mkcert`.
For step-by-step certificate creation and Kubernetes secret configuration, refer to **[docs/mkcert-setup.md](mkcert-setup.md)**.
