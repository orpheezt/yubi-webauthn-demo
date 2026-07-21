# Product Definition

## Project Title
Yubi WebAuthn Demo

## Vision & Summary
A full-stack WebAuthn demonstration application. The backend provides a secure REST API for credential management, while the frontend offers a user interface for testing WebAuthn flows using Shadcn UI. Both components are containerized for deployment on Kubernetes, with their respective manifests residing in component-specific `k8s/` directories. The backend utilizes a PostgreSQL database managed by the CloudNativePG operator. Additionally, a top-level `scripts/` directory orchestrates local deployment and testing via Minikube.

## Target Audience
- Developers looking to understand and implement WebAuthn flows.
- Security professionals testing FIDO2 implementations.

## Core Features
- Secure credential management via REST API.
- End-to-end WebAuthn flow testing interface.
- Kubernetes-native deployment (CloudNativePG, Rancher local-path provisioner, component-specific manifests).
- Local Minikube deployment & cluster lifecycle management (`make up`, `make status`, `make verify`, `make down`, `make clean`).
