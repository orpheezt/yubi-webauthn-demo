#!/bin/bash
set -e

echo "=> Building backend image..."
buildah bud -t yubi-backend:latest -f backend/Containerfile backend/

echo "=> Building migrations image..."
buildah bud -t yubi-migrations:latest -f backend/migrations.Containerfile backend/

echo "=> Building frontend image..."
buildah bud -t yubi-frontend:latest -f frontend/Containerfile frontend/

echo "=> Loading images to Minikube..."
# Alternative 1 (Current): Direct push to Minikube's internal Docker daemon
buildah push yubi-backend:latest docker-daemon:yubi-backend:latest
buildah push yubi-migrations:latest docker-daemon:yubi-migrations:latest
buildah push yubi-frontend:latest docker-daemon:yubi-frontend:latest

# Alternative 2 (Tar archive): Export and load via minikube
# buildah push yubi-backend:latest docker-archive:backend.tar && minikube image load backend.tar
# buildah push yubi-migrations:latest docker-archive:migrations.tar && minikube image load migrations.tar
# buildah push yubi-frontend:latest docker-archive:frontend.tar && minikube image load frontend.tar

# Alternative 3 (Local Registry): Push to the minikube registry addon
# buildah push --tls-verify=false yubi-backend:latest localhost:5000/yubi-backend:latest

echo "=> Creating 'yubi' namespace..."
kubectl create namespace yubi --dry-run=client -o yaml | kubectl apply -f -

echo "=> Applying Kubernetes manifests..."
kubectl apply -n yubi -f k8s/postgres.yaml
# wait for postgres to be ready
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
