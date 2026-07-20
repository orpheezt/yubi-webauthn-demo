#!/bin/bash
set -e

echo "=> Building backend image..."
buildah bud -t localhost/yubi-backend:latest -f backend/Containerfile backend/

echo "=> Building migrations image..."
buildah bud -t localhost/yubi-migrations:latest -f backend/migrations.Containerfile backend/

echo "=> Building frontend image..."
buildah bud -t localhost/yubi-frontend:latest -f frontend/Containerfile frontend/

echo "=> Pushing images to Minikube..."
# buildah push directly to minikube docker daemon, or save to tar and load
buildah push localhost/yubi-backend:latest docker-daemon:localhost/yubi-backend:latest
buildah push localhost/yubi-migrations:latest docker-daemon:localhost/yubi-migrations:latest
buildah push localhost/yubi-frontend:latest docker-daemon:localhost/yubi-frontend:latest

echo "=> Applying Kubernetes manifests..."
kubectl apply -f k8s/postgres.yaml
# wait for postgres to be ready
echo "=> Waiting for CloudNativePG cluster to be ready..."
kubectl wait --for=condition=Ready cluster/yubi-pg --timeout=300s || true

kubectl apply -f k8s/migrations.yaml
echo "=> Waiting for migrations to complete..."
kubectl wait --for=condition=complete job/yubi-migrations --timeout=120s || true

kubectl apply -f k8s/backend.yaml
kubectl apply -f k8s/frontend.yaml
kubectl apply -f k8s/ingress.yaml

echo "=> Deployment complete!"
echo "Ensure 'yubi.local' is mapped to your minikube IP in /etc/hosts"
