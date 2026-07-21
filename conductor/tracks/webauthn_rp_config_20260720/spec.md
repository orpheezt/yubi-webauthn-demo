# Spec: Fix WebAuthn RP Config & Cilium Gateway Infrastructure

## Overview

The WebAuthn Relying Party configuration in the Kubernetes backend ConfigMap (`k8s/backend.yaml`) was hardcoded to `localhost` development defaults. When accessing the application through the Cilium Gateway (hostname `yubi.local`), the browser sends the actual origin in the WebAuthn assertion, which didn't match the configured `RP_ORIGIN`. Furthermore, browsers strictly require a Secure Context (HTTPS or `localhost`) for WebAuthn APIs (`navigator.credentials`), necessitating HTTPS termination on the gateway using `mkcert` trusted certificates.

---

## Root Cause

- `RP_ID` was set to `"localhost"` instead of `"yubi.local"`.
- `RP_ORIGIN` was set to `"http://localhost:3000"` instead of `"https://yubi.local"`.
- Plain HTTP access on custom domain `yubi.local` disabled browser WebAuthn security context (`NotAllowedError`).

---

## Step-by-Step Setup Guide

### 1. DNS Mapping Setup (`yubi.local`)

1. Obtain the LoadBalancer IP assigned to `yubi-gateway`:
   ```bash
   kubectl get gateway yubi-gateway -n yubi
   # Output: ADDRESS: 192.168.39.240
   ```
2. Map `yubi.local` to the Gateway IP in `/etc/hosts`:
   ```bash
   echo "192.168.39.240 yubi.local" | sudo tee -a /etc/hosts
   ```

---

### 2. HTTPS Certificate Setup (`mkcert`)

1. **Install `mkcert` & `nss-tools`** (Fedora Linux):
   ```bash
   sudo dnf install -y mkcert nss-tools
   ```
2. **Install Local Root CA into OS & Browser Trust Stores**:
   ```bash
   mkcert -install
   ```
3. **Generate Certificate for `yubi.local`**:
   ```bash
   mkdir -p /tmp/yubi-certs
   mkcert -key-file /tmp/yubi-certs/tls.key -cert-file /tmp/yubi-certs/tls.crt yubi.local
   ```
4. **Create Kubernetes TLS Secrets**:
   ```bash
   # Application namespace secret (used by yubi-gateway)
   kubectl create secret tls yubi-tls-secret \
     --key=/tmp/yubi-certs/tls.key \
     --cert=/tmp/yubi-certs/tls.crt \
     -n yubi --dry-run=client -o yaml | kubectl apply -f -

   # Cilium Operator secret sync namespace (used by Cilium Envoy proxy)
   kubectl create namespace cilium-secrets --dry-run=client -o yaml | kubectl apply -f -
   kubectl create secret tls yubi-yubi-tls-secret \
     --key=/tmp/yubi-certs/tls.key \
     --cert=/tmp/yubi-certs/tls.crt \
     -n cilium-secrets --dry-run=client -o yaml | kubectl apply -f -
   ```
5. **Access Application**:
   Open **[https://yubi.local](https://yubi.local)** in your browser. The connection is fully trusted and WebAuthn functions out-of-the-box.

---

## Infrastructure Provided by Cilium

The Kubernetes cluster utilizes the **official Cilium Helm chart (`cilium/cilium`)** to provide standard, production-grade cloud-native infrastructure:

1. **Cilium CNI (eBPF-based Networking)**:
   - High-performance pod-to-pod and node-to-node eBPF datapath routing (`veth` / `vxlan`).
   - eBPF host firewall and socket-level load balancing.

2. **Cilium Kube-Proxy Replacement (KPR)**:
   - Completely replaces standard `kube-proxy` iptables rules with eBPF TC (Traffic Control) and socket programs for fast load balancing.

3. **Cilium Gateway API Controller**:
   - Implements the Kubernetes Gateway API standard (`gateway.networking.k8s.io/v1`).
   - Translates `Gateway` and `HTTPRoute` objects into `CiliumEnvoyConfig` (CEC) custom resources.

4. **Cilium Envoy L7 Proxy**:
   - Containerized Envoy proxy daemonset (`cilium-envoy`) handling TLS termination, HTTP header modification, and L7 path-based routing.

5. **Cilium LB-IPAM (LoadBalancer IP Address Management)**:
   - Manages LoadBalancer IP allocation via `CiliumLoadBalancerIPPool` (`cilium.io/v2`), assigning IP `192.168.39.240` from the node subnet.

6. **Cilium L2 Announcements**:
   - ARP-based Layer-2 announcement policy (`CiliumL2AnnouncementPolicy` `cilium.io/v2alpha1`) broadcasting LoadBalancer VIPs on `eth0`.

---

## Enabled Services & Architecture Topology

| Service Name | Namespace | Type | Address / Port | Role |
|---|---|---|---|---|
| **`cilium-gateway-yubi-gateway`** | `yubi` | `LoadBalancer` | `192.168.39.240:80, 443` | Cilium Gateway API L7 Proxy & HTTPS TLS Termination |
| **`yubi-frontend`** | `yubi` | `ClusterIP` | `10.104.193.215:3000` | TanStack Start / Nitro React SSR/SPA Frontend Server |
| **`yubi-backend`** | `yubi` | `ClusterIP` | `10.111.75.103:8080` | Rust / Axum REST API & WebAuthn Relying Party |
| **`yubi-pg-rw`** | `yubi` | `ClusterIP` | `10.111.195.207:5432` | CloudNativePG Primary PostgreSQL Database |
| **`yubi-pg-ro`** | `yubi` | `ClusterIP` | `10.106.88.212:5432` | CloudNativePG Read-Only PostgreSQL Replica |
| **`yubi-pg-r`** | `yubi` | `ClusterIP` | `10.97.36.150:5432` | CloudNativePG Read PostgreSQL Service |
| **`cilium-operator`** | `kube-system` | `Deployment` | Internal | Gateway API & CRD controller, Secret synchronizer |
| **`cilium-envoy`** | `kube-system` | `DaemonSet` | Host / `9999` | Envoy proxy datapath workloads |

---

## Acceptance Criteria

1. `k8s/backend.yaml` ConfigMap contains `RP_ID: "yubi.local"` and `RP_ORIGIN: "https://yubi.local"`.
2. WebAuthn passkey registration and authentication succeed over `https://yubi.local`.
3. Cilium Gateway API is fully programmed (`PROGRAMMED: True`) and routing HTTPS traffic to frontend and backend services.
