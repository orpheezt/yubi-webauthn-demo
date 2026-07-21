#!/bin/bash
set -e

echo "=> Starting temporary PostgreSQL database for build..."
if command -v podman >/dev/null 2>&1; then CONTAINER_CMD="podman"; else CONTAINER_CMD="docker"; fi
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

echo "=> Loading images to Minikube..."
rm -f backend.tar.zst migrations.tar.zst frontend.tar.zst
buildah push --compression-format zstd yubi-backend:latest oci-archive:backend.tar.zst && minikube image load backend.tar.zst
buildah push --compression-format zstd yubi-migrations:latest oci-archive:migrations.tar.zst && minikube image load migrations.tar.zst
buildah push --compression-format zstd yubi-frontend:latest oci-archive:frontend.tar.zst && minikube image load frontend.tar.zst

echo "=> Installing CloudNativePG operator via Helm..."
helm repo add cnpg https://cloudnative-pg.github.io/charts
helm repo update
helm upgrade --install cnpg cnpg/cloudnative-pg --namespace cnpg-system --create-namespace
kubectl wait --for=condition=Ready pod -l app.kubernetes.io/name=cloudnative-pg -n cnpg-system --timeout=300s || true

echo "=> Creating 'yubi' namespace..."
kubectl create namespace yubi --dry-run=client -o yaml | kubectl apply -f -

echo "=> Applying Kubernetes manifests..."
kubectl apply -n yubi -f k8s/postgres.yaml

echo "=> Waiting for CloudNativePG cluster to be ready..."
kubectl wait -n yubi --for=condition=Ready cluster/yubi-pg --timeout=600s || true

kubectl apply -n yubi -f k8s/migrations.yaml
echo "=> Waiting for migrations to complete..."
kubectl wait -n yubi --for=condition=complete job/yubi-migrations --timeout=300s || true

kubectl apply -n yubi -f k8s/backend.yaml
kubectl apply -n yubi -f k8s/frontend.yaml
kubectl apply -n yubi -f k8s/gateway.yaml

echo "=> Deployment complete!"
echo "Ensure 'yubi.local' is mapped to your minikube IP in /etc/hosts"
