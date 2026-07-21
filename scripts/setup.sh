#!/usr/bin/env bash
set -eo pipefail

echo "======================================================"
echo "      Yubi WebAuthn Cluster Automated Setup           "
echo "======================================================"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "[1/6] Checking Minikube status..."
if ! command -v minikube &> /dev/null; then
    echo "ERROR: minikube CLI is not installed or not in PATH."
    exit 1
fi

if ! minikube status &> /dev/null; then
    echo "Minikube is not running. Starting Minikube cluster..."
    minikube start --addons=storage-provisioner=false,default-storageclass=false
else
    echo "Minikube is running."
fi

echo "[2/6] Installing Rancher Local-Path Provisioner..."
kubectl delete sc standard --ignore-not-found
kubectl apply -f k8s/rancher-local-path.yaml
kubectl annotate storageclass standard storageclass.kubernetes.io/is-default-class=true --overwrite
echo "Waiting for Rancher local-path-provisioner to be ready..."
kubectl rollout status deployment/local-path-provisioner -n local-path-storage --timeout=90s

echo "[3/6] Building and loading container images..."
if command -v podman >/dev/null 2>&1; then CONTAINER_CMD="podman"; else CONTAINER_CMD="docker"; fi

echo "=> Starting temporary PostgreSQL database for build..."
$CONTAINER_CMD run -d --name deploy-db-build -p 5432:5432 -e POSTGRES_PASSWORD=secret docker.io/library/postgres:18.4-trixie
sleep 5
export DATABASE_URL="postgres://postgres:secret@127.0.0.1:5432/postgres"
$CONTAINER_CMD run --rm --network host -e PGPASSWORD=secret -v "$PWD/migrations:/migrations" docker.io/library/postgres:18.4-trixie sh -c 'for f in /migrations/*.up.sql; do psql -h 127.0.0.1 -U postgres -d postgres -f "$f"; done'

echo "=> Building backend image..."
buildah bud --network=host --build-arg DATABASE_URL=$DATABASE_URL -t yubi-backend:latest -f backend/Containerfile backend/
$CONTAINER_CMD rm -f deploy-db-build

echo "=> Building migrations image..."
buildah bud -t yubi-migrations:latest -f backend/migrations.Containerfile .

echo "=> Building frontend image..."
buildah bud -t yubi-frontend:latest -f frontend/Containerfile frontend/

echo "=> Loading images into Minikube..."
rm -f backend.tar.zst migrations.tar.zst frontend.tar.zst
buildah push --compression-format zstd yubi-backend:latest oci-archive:backend.tar.zst && minikube image load backend.tar.zst
buildah push --compression-format zstd yubi-migrations:latest oci-archive:migrations.tar.zst && minikube image load migrations.tar.zst
buildah push --compression-format zstd yubi-frontend:latest oci-archive:frontend.tar.zst && minikube image load frontend.tar.zst

echo "[4/6] Installing CloudNativePG Operator via Helm..."
helm repo add cnpg https://cloudnative-pg.github.io/charts || true
helm repo update
helm upgrade --install cnpg cnpg/cloudnative-pg --namespace cnpg-system --create-namespace
echo "Waiting for CloudNativePG operator deployment..."
kubectl wait --for=condition=Ready pod -l app.kubernetes.io/name=cloudnative-pg -n cnpg-system --timeout=120s || true

echo "[5/6] Deploying Yubi Application Manifests & Database Cluster..."
kubectl create namespace yubi --dry-run=client -o yaml | kubectl apply -f -
kubectl apply -n yubi -f k8s/postgres.yaml

echo "Waiting for CloudNativePG database cluster (yubi-pg)..."
kubectl wait -n yubi --for=condition=Ready cluster/yubi-pg --timeout=300s || true

kubectl apply -n yubi -f k8s/migrations.yaml
echo "Waiting for migrations job completion..."
kubectl wait -n yubi --for=condition=complete job/yubi-migrations --timeout=120s || true

kubectl apply -n yubi -f k8s/backend.yaml
kubectl apply -n yubi -f k8s/frontend.yaml
kubectl apply -n yubi -f k8s/gateway.yaml

echo "======================================================"
echo "          Setup Completed Successfully!               "
echo "======================================================"
echo "Ensure 'yubi.local' is mapped to your minikube IP in /etc/hosts:"
echo "  echo \"\$(minikube ip) yubi.local\" | sudo tee -a /etc/hosts"
echo ""
echo "Cluster Status:"
"$SCRIPT_DIR/status.sh"
