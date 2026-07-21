#!/usr/bin/env bash
set -eo pipefail

MODE="${1:-down}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

if [ "$MODE" = "down" ]; then
    echo "======================================================"
    echo "       Teardown: Removing Yubi Application            "
    echo "======================================================"
    kubectl delete namespace yubi --ignore-not-found
    echo "Application namespace 'yubi' removed successfully."

elif [ "$MODE" = "clean" ]; then
    echo "======================================================"
    echo "       Full Clean: Removing App, Operators & Provisioner "
    echo "======================================================"
    echo "[1/4] Removing Yubi application namespace..."
    kubectl delete namespace yubi --ignore-not-found

    echo "[2/4] Uninstalling CloudNativePG Operator..."
    helm uninstall cnpg -n cnpg-system 2>/dev/null || true
    kubectl delete namespace cnpg-system --ignore-not-found

    echo "[3/4] Removing Rancher Local-Path Provisioner..."
    kubectl delete -f k8s/rancher-local-path.yaml --ignore-not-found
    kubectl delete namespace local-path-storage --ignore-not-found

    echo "[4/4] Removing local build tarballs..."
    rm -f backend.tar.zst migrations.tar.zst frontend.tar.zst

    echo "Full cleanup completed successfully."
else
    echo "Usage: $0 [down|clean]"
    exit 1
fi
