# envArchitect

## Project Structure

```
envArchitect/
├── apps/
│   ├── docs/         # Documentation application (Next.js)
│   └── cli/          # Command-line interface (Rust)
├── packages/
│   ├── env-architect/  # envArchitect core
│   └── env/ # environment
│   └── config/ # configurations
```

## Available Scripts

- `npm run dev`: Start all applications in development mode
- `npm run build`: Build all applications
- `npm run dev:web`: Start only the web application
- `npm run check-types`: Check TypeScript types across all apps
- `npm run check`: Run Biome formatting and linting
- `cd apps/web && npm run generate-pwa-assets`: Generate PWA assets
