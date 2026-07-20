#!/bin/bash
set -e

echo "=> Building backend image..."
buildah bud -t yubi-backend:latest -f backend/Containerfile backend/

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
kubectl wait --for=condition=Ready pod -l app.kubernetes.io/name=cloudnative-pg -n cnpg-system --timeout=120s || true

echo "=> Creating 'yubi' namespace..."
kubectl create namespace yubi --dry-run=client -o yaml | kubectl apply -f -

echo "=> Applying Kubernetes manifests..."
kubectl apply -n yubi -f k8s/postgres.yaml

echo "=> Waiting for CloudNativePG cluster to be ready..."
kubectl wait -n yubi --for=condition=Ready cluster/yubi-pg --timeout=300s || true

kubectl apply -n yubi -f k8s/migrations.yaml
echo "=> Waiting for migrations to complete..."
kubectl wait -n yubi --for=condition=complete job/yubi-migrations --timeout=120s || true

kubectl apply -n yubi -f k8s/backend.yaml
kubectl apply -n yubi -f k8s/frontend.yaml
kubectl apply -n yubi -f k8s/gateway.yaml

echo "=> Deployment complete!"
echo "Ensure 'yubi.local' is mapped to your minikube IP in /etc/hosts"
