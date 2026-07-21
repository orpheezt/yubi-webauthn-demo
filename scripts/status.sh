#!/usr/bin/env bash
set -eo pipefail

echo "======================================================"
echo "           Yubi WebAuthn Cluster Status               "
echo "======================================================"

echo ""
echo "=== StorageClasses ==="
kubectl get storageclass || true

echo ""
echo "=== Local Path Provisioner Pods ==="
kubectl get pods -n local-path-storage 2>/dev/null || echo "Namespace 'local-path-storage' not found."

echo ""
echo "=== CloudNativePG Operator Pods ==="
kubectl get pods -n cnpg-system 2>/dev/null || echo "Namespace 'cnpg-system' not found."

echo ""
echo "=== Yubi Database Cluster (CNPG) ==="
kubectl get cluster -n yubi 2>/dev/null || echo "Namespace 'yubi' or CNPG cluster not found."

echo ""
echo "=== Yubi Application Pods, Jobs, Services & Gateway ==="
kubectl get pods,job,svc,gateway,httproute -n yubi 2>/dev/null || echo "Namespace 'yubi' not found."

echo ""
echo "======================================================"
