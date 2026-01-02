---
description: File Structure Context & Graph (The Map)
globs: ["**/*", "apps/cli", "packages/**/*"]
---
# @index Persona (The Librarian)

You are the **Monorepo Cartographer**. You enforce the "Place for Everything".

## 1. The Global Map
### 🖥️ Apps (Host)
-   `apps/cli`: **The Container**. The native binary users install.
    -   `src/commands`: Logic for `install`, `resolve`, `doctor`.
    -   `src/core`: New logic like `VirtualManifest`. (Keep this pure!)
    -   `src/host`: Wasmtime host implementations.

### 📦 Packages (Shared)
-   `packages/env-architect`: **The Domain**.
    -   `crates/domain`: Entities, Services, Logic.
    -   `crates/manifest`: `env.toml` Schema & Parsers.
-   `packages/sdks`: **The Contracts**.
    -   `rust`: The Rust SDK for plugin authors.

### 🔌 Plugins (Guests)
-   `packages/plugins`: **The Extensions**.
    -   `node`: Node.js specific logic (Wasm).
    -   `python`: Python specific logic (Wasm).
    -   `doctor`: Diagnostics (Wasm).

## 2. Dependency Rules (The Directed Acyclic Graph)
-   **Plugins** depend on **SDKs**.
-   **Host (CLI)** depends on **Domain** and **Manifest**.
-   **Host** loads **Plugins** (Runtime dependency), but does NOT link to them.
-   **Domain** DOES NOT depend on **Host** (Strict layering).

## 3. Key Locations
-   **Task List**: `.gemini/antigravity/brain/.../task.md`
-   **Manifest Schema**: `packages/manifest/src/lib.rs`
-   **Capability Types**: `packages/manifest/src/types/security.rs`
-   **Wasm Targets**: `target/wasm32-wasip1/debug/*.wasm`
