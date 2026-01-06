#!/bin/bash

# Ensure we are in the project root
cd "$(dirname "$0")/.."

echo "🚀 Starting Environment Services..."

# 1. Start Infrastructure (Background)
echo "🐘 Starting Postgres & MinIO..."
docker-compose up -d
# Wait for DB to be healthy (simple sleep for now, could be robust)
sleep 5

# 2. Migration
# echo "🔄 Running Migrations..."
# cargo sqlx migrate run --source packages/server/database/src/migrations

# 3. Start Services Concurrently
echo "⚡ Starting API, Worker, and Web..."
# We use a trap to kill all child processes on Ctrl+C
trap 'kill $(jobs -p)' SIGINT

# Start API
(cd packages/server/api && cargo run) &
PID_API=$!

# Start Worker
(cd packages/server/worker && cargo run) &
PID_WORKER=$!

# Start Web
(cd apps/web && bun dev) &
PID_WEB=$!

wait $PID_API $PID_WORKER $PID_WEB
