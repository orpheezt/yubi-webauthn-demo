#!/usr/bin/env bash
set -eo pipefail

FAILED=0

echo "======================================================"
echo "          Yubi WebAuthn Verification Check            "
echo "======================================================"

echo "[1/6] Verifying StorageClass..."
DEFAULT_SC=$(kubectl get sc -o jsonpath='{.items[?(@.metadata.annotations.storageclass\.kubernetes\.io/is-default-class=="true")].metadata.name}' 2>/dev/null || true)
if [ "$DEFAULT_SC" = "standard" ]; then
    echo "PASS: StorageClass 'standard' (via Rancher local-path provisioner) is default."
else
    echo "FAIL: Default StorageClass is '$DEFAULT_SC' (expected 'standard')."
    FAILED=1
fi

echo "[2/5] Verifying Local Path Provisioner..."
if kubectl rollout status deployment/local-path-provisioner -n local-path-storage --timeout=30s &>/dev/null; then
    echo "PASS: Rancher local-path-provisioner deployment is ready."
else
    echo "FAIL: Rancher local-path-provisioner deployment is not ready."
    FAILED=1
fi

echo "[3/5] Verifying CloudNativePG Operator..."
if kubectl rollout status deployment/cnpg-cloudnative-pg -n cnpg-system --timeout=30s &>/dev/null; then
    echo "PASS: CloudNativePG operator deployment is ready."
else
    echo "FAIL: CloudNativePG operator deployment is not ready."
    FAILED=1
fi

echo "[4/5] Verifying Database Cluster (yubi-pg)..."
if kubectl wait -n yubi --for=condition=Ready cluster/yubi-pg --timeout=30s &>/dev/null; then
    echo "PASS: Database cluster 'yubi-pg' is ready."
else
    echo "FAIL: Database cluster 'yubi-pg' is not ready."
    FAILED=1
fi

echo "[5/5] Verifying Application Deployments & Jobs..."
if kubectl wait -n yubi --for=condition=complete job/yubi-migrations --timeout=30s &>/dev/null; then
    echo "PASS: Migration job 'yubi-migrations' completed successfully."
else
    echo "FAIL: Migration job 'yubi-migrations' is not complete."
    FAILED=1
fi

if kubectl rollout status deployment/yubi-backend -n yubi --timeout=30s &>/dev/null; then
    echo "PASS: Backend deployment 'yubi-backend' is ready."
else
    echo "FAIL: Backend deployment 'yubi-backend' is not ready."
    FAILED=1
fi

echo "[6/6] Verifying Cilium Gateway API Configuration & Resources..."
GW_ENABLED=$(kubectl get configmap cilium-config -n kube-system -o jsonpath='{.data.enable-gateway-api}' 2>/dev/null || true)
if [ "$GW_ENABLED" = "true" ]; then
    echo "PASS: cilium-config has enable-gateway-api='true'."
else
    echo "FAIL: cilium-config enable-gateway-api is '$GW_ENABLED' (expected 'true')."
    FAILED=1
fi

if kubectl get crd gatewayclasses.gateway.networking.k8s.io &>/dev/null; then
    echo "PASS: Gateway API CRDs are installed."
else
    echo "FAIL: Gateway API CRDs are missing."
    FAILED=1
fi

if kubectl get gatewayclass cilium &>/dev/null; then
    echo "PASS: GatewayClass 'cilium' exists."
else
    echo "FAIL: GatewayClass 'cilium' does not exist."
    FAILED=1
fi

echo "======================================================"
if [ "$FAILED" -eq 0 ]; then
    echo "ALL VERIFICATION CHECKS PASSED!"
    exit 0
else
    echo "SOME VERIFICATION CHECKS FAILED."
    exit 1
fi
