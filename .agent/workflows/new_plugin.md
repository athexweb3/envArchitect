---
description: How to scaffold a new Wasm Plugin
---
# New Plugin Workflow

## 1. Scaffolding
-   **Directory**: `mkdir -p packages/plugins/<name>`.
-   **Init**: `cargo init --lib packages/plugins/<name>`.
-   **Type**: Set `crate-type = ["cdylib"]` in `Cargo.toml`.

## 2. Configuration
-   **Dependencies**:
    -   Add `env-architect-sdk` (path: `../../sdks/rust`).
    -   Add `wit-bindgen`.
-   **Component Metadata**:
    -   Add `[package.metadata.component]` section.
    -   Define `package = "env-architect:<name>"`.

## 3. Implementation
-   **WIT World**: Ensure usage of the `plugin` world from the SDK.
-   **Handler**: Implement the `PluginHandler` trait in `src/lib.rs`.
-   **Macro**: Annotate with `#[env_architect_macros::plugin]`.

## 4. Manifest
-   **Create `env.toml`**:
    -   `[project]` metadata.
    -   `[capabilities]`: List ONLY what is needed (Least Privilege).

## 5. Build & Test
-   **Build**: `cargo component build --release`.
-   **Register**: Add to `apps/cli` if it's a built-in, or test via `env-architect dev`.
