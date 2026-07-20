#!/bin/bash
set -e

if command -v podman >/dev/null 2>&1; then
    CONTAINER_CMD="podman"
elif command -v docker >/dev/null 2>&1; then
    CONTAINER_CMD="docker"
else
    echo "Neither podman nor docker found."
    exit 1
fi

echo "=> Starting temporary PostgreSQL database using $CONTAINER_CMD..."
$CONTAINER_CMD run -d --name sqlx-prepare -p 5432:5432 -e POSTGRES_PASSWORD=secret docker.io/library/postgres:16-alpine

echo "=> Waiting for database to be ready..."
sleep 5 # give it a moment to boot

export DATABASE_URL="postgres://postgres:secret@localhost:5432/postgres"

echo "=> Running migrations..."
cargo install sqlx-cli --no-default-features --features rustls,postgres || true
sqlx migrate run

echo "=> Running cargo sqlx prepare..."
cd backend
cargo sqlx prepare

echo "=> Cleaning up..."
cd ..
$CONTAINER_CMD rm -f sqlx-prepare
echo "=> Done! .sqlx metadata generated."
