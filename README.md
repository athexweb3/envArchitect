# EnvArchitect

**A declarative environment architecture tool for defining, versioning, and distributing development environments.**

EnvArchitect allows teams to define their entire development ecosystem ("Architecture") in a single `env.toml` file. It ensures that every developer has the exact same tools, versions, and configurations, functioning as an immutable source of truth for your environment.

## Core Mission
To treat development environments as **Architectural Artifacts**—designed, versioned, and secured just like your production infrastructure.

## Key Features
*   **Declarative Manifest (`env.toml`)**: Define tools, runtimes, and dependencies in one readable file.
*   **Immutable Environments**: "Works on my machine" becomes impossible.
*   **Extensible Plugin System**: While the core handles the architecture, third-party developers can extend capabilities via secure Wasm plugins.
*   **Secure Distribution**: A managed registry ensuring supply chain security for all environment tools.

## Architecture
### 1. The Core (`packages/env-architect`)
The intelligence engine that resolves dependency graphs and validates environment architecture.

### 2. The Tooling
*   **CLI (`apps/cli`)**: The primary interface to architect and deploy environments.
*   **VS Code Extension**: Real-time intelligence and validation for your `env.toml` architecture.

### 3. The Platform
*   **Registry API**: Central hub for storing and distributing environment definitions and plugins.
*   **Web Portal**: Dashboard for managing team environments and discovering plugins.

## Tech Stack
*   **Rust**: For high-performance, safe core logic and tooling.
*   **TypeScript**: For modern web interfaces and editor integrations.
*   **WebAssembly**: For a sandboxed, secure plugin runtime.

## License
MIT
