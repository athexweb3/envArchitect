# Security Policy

EnvArchitect takes security seriously. Our mission is to provide an immutable, secure environment for development, and we treat the security of our tool, registry, and plugin ecosystem as a top priority.

## Supported Versions

We currently support security updates for the latest major version.

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1.0 | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability within EnvArchitect, please do **not** disclose it publicly until it has been addressed.

**How to report:**
Please use the [GitHub Private Vulnerability Reporting](https://github.com/athexweb3/envArchitect/security/advisories/new) feature to report vulnerabilities securely.

If that is not possible, you may email `athexweb3@gmail.com`.

We commit to acknowledging your report as soon as possible and will provide regular updates on our remediation progress.

## Threat Model & Architecture

EnvArchitect is designed with a "Secure by Default" philosophy. We employ several architectural defenses to protect user environments.

### 1. Wasm Plugin Sandbox
All third-party plugins run inside a strictly sandboxed WebAssembly runtime (**Wasmtime**). Unlike native scripts (npm post-install, shell scripts) which have full system access, EnvArchitect plugins:
*   **Cannot** access the filesystem.
*   **Cannot** access the network.
*   **Cannot** read environment variables.
*   **Cannot** spawn child processes.

...unless explicitly granted a **Capability** by the user.

### 2. Capability-Based Security
EnvArchitect implements a strict capability model (`packages/manifest/src/types/security.rs`). Plugins must declare required permissions in their manifest.

Example of enforced capabilities:
*   `fs-read`: Read-only access to specific paths.
*   `network`: Outbound access to specific hosts (e.g., `github.com`).
*   `ui-secret`: Permission to mask user input for sensitive data.

The core runtime (`SystemExecutor`) enforces these boundaries at the system call level.

### 3. Supply Chain Security (Planned)
We are integrating **The Update Framework (TUF)** and **Sigstore** to ensure that all artifacts (plugins, tools) are signed and verified before execution. This prevents tampering and man-in-the-middle attacks during the environment resolution phase.

## Scope

This policy applies to:
*   The EnvArchitect CLI (`apps/cli`)
*   The Core Library (`packages/env-architect`)
*   The Official Plugin SDKs (`packages/sdks`)
*   The Official Registry API (`apps/api`)

## Out of Scope
*   Vulnerabilities in third-party tools installed *by* EnvArchitect (e.g., a vulnerability in `node` itself), though we facilitate updating to secure versions.
*   Malicious usage of a user's own capabilities (e.g., if a user explicitly grants `fs-write` to `/` to an untrusted plugin).
