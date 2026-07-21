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
    echo "Minikube is not running."
    MINIKUBE_CMD="minikube start --nodes=4 --driver=kvm2 --cni=cilium --cpus=2 --memory=4096 --disk-size=20g --kubernetes-version=v1.36.1 --container-runtime=containerd --addons=metrics-server"
    echo "======================================================"
    echo "Minikube Start Command:"
    echo "  $MINIKUBE_CMD"
    echo "======================================================"
    if [ -t 0 ] && [ "${AUTO_APPROVE:-false}" != "true" ]; then
        YESEXPR=$(locale yesexpr 2>/dev/null || echo "^[yY]")
        read -p "Proceed with starting Minikube? [y/N] " confirm
        if ! [[ "$confirm" =~ $YESEXPR ]]; then
            echo "Operation cancelled by user."
            exit 1
        fi
    fi
    $MINIKUBE_CMD
    echo "Disabling default storage addons in favor of Rancher local-path-provisioner..."
    minikube addons disable storage-provisioner || true
    minikube addons disable default-storageclass || true
else
    echo "Minikube is running."
fi

echo "=> Enabling Gateway API on Cilium CNI..."
chmod +x "$SCRIPT_DIR/cilium_gateway.sh"
"$SCRIPT_DIR/cilium_gateway.sh"

echo "[2/6] Installing Rancher Local-Path Provisioner..."
kubectl delete sc standard --ignore-not-found
kubectl apply -f k8s/rancher-local-path.yaml
kubectl annotate storageclass standard storageclass.kubernetes.io/is-default-class=true --overwrite
echo "Waiting for Rancher local-path-provisioner to be ready..."
kubectl rollout status deployment/local-path-provisioner -n local-path-storage --timeout=300s

echo "[3/6] Building and loading container images..."
if command -v podman >/dev/null 2>&1; then CONTAINER_CMD="podman"; else CONTAINER_CMD="docker"; fi

echo "=> Starting temporary PostgreSQL database for build..."
$CONTAINER_CMD rm -f deploy-db-build >/dev/null 2>&1 || true
$CONTAINER_CMD run -d --name deploy-db-build -p 5432:5432 -e POSTGRES_PASSWORD=secret docker.io/library/postgres:18.4-trixie
echo "Waiting for temporary PostgreSQL to be ready..."
for i in {1..30}; do
    if $CONTAINER_CMD exec deploy-db-build pg_isready -U postgres >/dev/null 2>&1; then
        break
    fi
    sleep 1
done
export DATABASE_URL="postgres://postgres:secret@127.0.0.1:5432/postgres"
$CONTAINER_CMD run --rm --network host -e PGPASSWORD=secret -v "$PWD/migrations:/migrations:z" docker.io/library/postgres:18.4-trixie sh -c 'for f in /migrations/*.up.sql; do psql -h 127.0.0.1 -U postgres -d postgres -f "$f"; done'

echo "=> Building backend image..."
buildah bud --network=host --build-arg DATABASE_URL=$DATABASE_URL -t yubi-backend:latest -f backend/Containerfile backend/
$CONTAINER_CMD rm -f deploy-db-build

echo "=> Building migrations image..."
buildah bud -t yubi-migrations:latest -f backend/migrations.Containerfile .

echo "=> Building frontend image..."
buildah bud -t yubi-frontend:latest -f frontend/Containerfile frontend/

echo "=> Loading images into Minikube..."
rm -f backend.tar migrations.tar frontend.tar backend.tar.zst migrations.tar.zst frontend.tar.zst
$CONTAINER_CMD image save -o backend.tar yubi-backend:latest && minikube image load backend.tar && rm -f backend.tar
$CONTAINER_CMD image save -o migrations.tar yubi-migrations:latest && minikube image load migrations.tar && rm -f migrations.tar
$CONTAINER_CMD image save -o frontend.tar yubi-frontend:latest && minikube image load frontend.tar && rm -f frontend.tar

echo "[4/6] Installing CloudNativePG Operator via Helm..."
helm repo add cnpg https://cloudnative-pg.github.io/charts || true
helm repo update
helm upgrade --install cnpg cnpg/cloudnative-pg \
  --version 0.29.0 \
  --namespace cnpg-system \
  --create-namespace
echo "Waiting for CloudNativePG operator deployment..."
kubectl wait --for=condition=Ready pod -l app.kubernetes.io/name=cloudnative-pg -n cnpg-system --timeout=300s || true

echo "[5/6] Deploying Yubi Application Manifests & Database Cluster..."
kubectl create namespace yubi --dry-run=client -o yaml | kubectl apply -f -
kubectl apply -n yubi -f k8s/postgres.yaml

echo "Waiting for CloudNativePG database cluster (yubi-pg)..."
kubectl wait -n yubi --for=condition=Ready cluster/yubi-pg --timeout=600s || true

kubectl apply -n yubi -f k8s/migrations.yaml
echo "Waiting for migrations job completion..."
kubectl wait -n yubi --for=condition=complete job/yubi-migrations --timeout=300s || true

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
