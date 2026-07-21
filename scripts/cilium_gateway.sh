#!/usr/bin/env bash
set -eo pipefail

echo "======================================================"
echo "    Cilium Gateway API & cert-manager Setup           "
echo "======================================================"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "[1/4] Deploying Cilium via Helm using k8s/cilium-values.yaml (with --wait)..."
helm repo add cilium https://helm.cilium.io/ 2>/dev/null || true
helm repo update cilium

# Remove minikube's initial static Cilium objects so Helm gets clean resource ownership
kubectl delete daemonset cilium cilium-envoy -n kube-system --ignore-not-found 2>/dev/null || true
kubectl delete deployment cilium-operator -n kube-system --ignore-not-found 2>/dev/null || true
kubectl delete configmap cilium-config cilium-envoy-config -n kube-system --ignore-not-found 2>/dev/null || true
kubectl delete clusterrole cilium cilium-operator --ignore-not-found 2>/dev/null || true

# Adopt cilium-secrets namespace for Helm ownership (prevents kubernetes finalizer hangs)
kubectl create namespace cilium-secrets --dry-run=client -o yaml | kubectl apply -f -
kubectl label namespace cilium-secrets app.kubernetes.io/managed-by=Helm --overwrite 2>/dev/null || true
kubectl annotate namespace cilium-secrets meta.helm.sh/release-name=cilium meta.helm.sh/release-namespace=kube-system --overwrite 2>/dev/null || true

K8S_HOST=$(kubectl config view --minify -o jsonpath='{.clusters[0].cluster.server}' | sed 's|https://||' | cut -d: -f1)
K8S_PORT=$(kubectl config view --minify -o jsonpath='{.clusters[0].cluster.server}' | sed 's|https://||' | cut -d: -f2)

helm upgrade --install cilium cilium/cilium \
  --version 1.19.6 \
  --namespace kube-system \
  --take-ownership \
  -f k8s/cilium-values.yaml \
  --set k8sServiceHost="$K8S_HOST" \
  --set k8sServicePort="$K8S_PORT" \
  --wait \
  --timeout 20m

echo "[2/4] Applying Cilium LB-IPAM pool and L2 announcement policy..."
kubectl apply -f k8s/cilium-lb.yaml

echo "[3/4] Installing cert-manager v1.21.0 via Helm (with --wait)..."
helm repo add jetstack https://charts.jetstack.io 2>/dev/null || true
helm repo update jetstack
helm upgrade --install cert-manager jetstack/cert-manager \
  --version v1.21.0 \
  --namespace cert-manager \
  --create-namespace \
  --set crds.enabled=true \
  --wait \
  --timeout 20m

echo "[4/4] Applying cert-manager ClusterIssuer & Certificate..."
kubectl create namespace yubi --dry-run=client -o yaml | kubectl apply -f -
kubectl apply -f k8s/cert-manager-issuer.yaml

echo "Waiting for cert-manager to issue yubi-tls-secret..."
kubectl wait -n yubi --for=condition=Ready certificate/yubi-local-cert --timeout=300s || true

# Sync secret to cilium-secrets namespace for Envoy
kubectl create namespace cilium-secrets --dry-run=client -o yaml | kubectl apply -f - 2>/dev/null || true
kubectl get secret yubi-tls-secret -n yubi -o yaml 2>/dev/null | \
  sed 's/name: yubi-tls-secret/name: yubi-yubi-tls-secret/' | \
  sed 's/namespace: yubi/namespace: cilium-secrets/' | \
  kubectl apply -f - 2>/dev/null || true

echo "======================================================"
echo "    Cilium Gateway & cert-manager Setup Complete!     "
echo "======================================================"
