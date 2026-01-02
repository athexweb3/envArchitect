---
description: How to run the Stress Test Suite
---
# Stress Test Workflow

## 1. Setup
-   **Context**: The stress test lives in `examples/feature-stress-plugin`.
-   **Requirement**: You must have `wasm-tools` and `cargo-component` installed.

## 2. Build Cycle
1.  **Build Plugin**:
    ```bash
    cd examples/feature-stress-plugin
    cargo component build
    ```
2.  **Verify Wasm**: Check `target/wasm32-wasip1/debug/*.wasm` exists.

## 3. Host Execution
1.  **Run CLI**:
    ```bash
    # From root
    cargo run -p env-architect-cli -- dev examples/feature-stress-plugin
    ```

## 4. Verification Steps
-   [ ] **UI Inputs**: Test `text`, `confirm`, `select`.
-   [ ] **Secrets**: Verify masked input works.
-   [ ] **Filesystem**: Verify it creates `test_output.txt`.
-   [ ] **Capabilities**: Verify it asks for permission if new caps are added.
