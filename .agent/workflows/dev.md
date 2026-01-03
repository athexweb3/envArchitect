---
description: Start all services (Database, Server, Frontend)
---
# Development Environment Workflow

## 🚀 Quick Start (Recommended)

Just run this ONE command to start everything:

// turbo-all
```bash
npm run dev:all
```

This will automatically:
1. ✅ Start PostgreSQL, Redis, MinIO (Docker)
2. ✅ Run database migrations
3. ✅ Start API server (port 3000)
4. ✅ Start frontend (port 3001)

## 📋 What's Running?

| Service | URL | Port |
|---------|-----|------|
| 🌐 Frontend | http://localhost:3001 | 3001 |
| 🔧 API Server | http://localhost:3000 | 3000 |
| 🗄️ Database | localhost:5432 | 5432 |
| 💾 Redis | localhost:6379 | 6379 |
| 📦 MinIO Console | http://localhost:9001 | 9001 |
| 📦 MinIO API | http://localhost:9000 | 9000 |

## 🛠️ Manual Steps (Optional)

If you want to start services individually:

### 1. Start Infrastructure Only

```bash
npm run dev:services
```

This starts Docker services + runs migrations.

### 2. Start Dev Servers Only

```bash
npm run dev
```

This uses Turbo to start both API and frontend in parallel.

### 3. Start Individual Services

**API Server Only:**
```bash
cd apps/api
npm run dev
```

**Frontend Only:**
```bash
cd apps/web
npm run dev
```

## 🛑 Stopping Services

**Stop all Docker services:**
```bash
docker compose down
```

**Stop with data cleanup:**
```bash
docker compose down -v
```

**Stop dev servers:**
Press `Ctrl+C` in the terminal running `npm run dev:all`

## 🔧 Troubleshooting

### Docker not starting

Make sure Docker Desktop is running:
```bash
open -a Docker
```

### Port already in use

Kill processes on specific ports:
```bash
# macOS/Linux
lsof -ti:3000 | xargs kill -9  # API
lsof -ti:3001 | xargs kill -9  # Frontend
```

### Database connection refused

Restart PostgreSQL:
```bash
docker compose restart postgres
```

Check if it's healthy:
```bash
docker compose ps postgres
```

### Migrations not running

Run manually:
```bash
cd packages/server/database
sqlx migrate run --source src/migrations
```

## 📦 Prerequisites

- **Docker Desktop** - For PostgreSQL, Redis, MinIO
- **Node.js** - For frontend (Next.js)
- **Rust** - For API server
- **sqlx-cli** - For database migrations

Install sqlx-cli if needed:
```bash
cargo install sqlx-cli --no-default-features --features postgres
```

## 🎯 Development Workflow

1. **First time setup:**
   ```bash
   npm install
   npm run dev:all
   ```

2. **Daily development:**
   ```bash
   npm run dev:all
   ```

3. **When done:**
   ```bash
   # Press Ctrl+C to stop servers
   docker compose down
   ```

That's it! 🎉

