# Technology Stack

## Backend
- **Language**: Rust
- **Framework**: Axum, Tokio
- **Database**: PostgreSQL (managed via SQLx)

## Frontend
- **Framework**: React 19, TanStack Start, Vite
- **Styling & UI**: Tailwind CSS, Shadcn UI

## Infrastructure & Deployment
- **Containerization**: Buildah
- **Orchestration**: Kubernetes
- **Local Deployment**: Minikube
- **Local Storage Provisioner**: Rancher Local-Path Provisioner
- **Lifecycle Management**: Makefile & modular lifecycle scripts (`scripts/setup.sh`, `scripts/teardown.sh`, `scripts/status.sh`, `scripts/verify.sh`)
- **Database Operator**: CloudNativePG
- **Database Migrations**: Executed via a dedicated Kubernetes Job using the `sqlx-cli` (Deviation recorded: 2026-07-19)
