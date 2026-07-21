#!/usr/bin/env bash
set -eo pipefail

echo "======================================================"
echo "    Cilium Gateway API Setup & Activation             "
echo "======================================================"

echo "[1/4] Installing Gateway API CRDs (v1.6.1)..."
kubectl apply -f https://github.com/kubernetes-sigs/gateway-api/releases/download/v1.6.1/standard-install.yaml

echo "[2/4] Patching cilium-config ConfigMap in kube-system..."
kubectl patch configmap cilium-config -n kube-system --type merge -p '{"data":{"enable-gateway-api":"true"}}'

echo "[3/4] Restarting Cilium DaemonSets and Operator Deployment..."
kubectl rollout restart daemonset/cilium -n kube-system
kubectl rollout restart daemonset/cilium-envoy -n kube-system 2>/dev/null || true
kubectl rollout restart deployment/cilium-operator -n kube-system

echo "[4/4] Waiting for Cilium rollouts to complete..."
kubectl rollout status daemonset/cilium -n kube-system --timeout=300s
kubectl rollout status deployment/cilium-operator -n kube-system --timeout=300s

echo "Cilium Gateway API setup complete."
