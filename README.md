# EnvArchitect

EnvArchitect is a declarative, immutable environment architecture tool for the modern web.

**To start using EnvArchitect**, learn more at [The Guide].

**To start developing EnvArchitect itself**, read the [Contributor Guide].

[The Guide]: https://github.com/athexweb3/envArchitect/tree/main/docs
[Contributor Guide]: CONTRIBUTING.md

> EnvArchitect bridges the gap between version managers (nvm, pyenv) and containerization
> (Docker). It orchestrates native tooling through a secure, hermetic layout controlled
> by a declarative graph.

## Code Status

[![CI](https://github.com/athexweb3/envArchitect/actions/workflows/main.yml/badge.svg)](https://github.com/athexweb3/envArchitect/actions/workflows/main.yml)

## Compiling from Source

### Requirements

EnvArchitect requires the following tools and packages to build:

* `cargo` and `rustc` (via rustup)
* A C compiler (gcc/clang) for certain native dependencies
* `git` (to clone this repository)

**Optional system libraries:**

The build system will automatically manage most dependencies. However, for optimized builds, you may provide:

* `openssl` — (via `pkg-config`) if you wish to link against the system OpenSSL instead of the vendored version.

### Compiling

First, you'll want to check out this repository

```bash
git clone https://github.com/athexweb3/envArchitect.git
cd envArchitect
```

With `cargo` already installed, you can simply run:

```bash
cargo build --release
```

The binary will be located in `target/release/env-architect`.

## Key Features

EnvArchitect is designed to provide a robust foundation for development teams.

*   **Declarative Manifest (`env.toml`)**: Define tools, runtimes, and dependencies in one readable file.
*   **Immutable Environments**: Ensures that "Works on my machine" is guaranteed by strictly locking tool versions.
*   **Extensible Plugin System**: Third-party developers can extend capabilities via secure Wasm plugins.
*   **Secure Distribution**: A managed registry ensuring supply chain security for all environment tools.

## Architecture

The project is organized as a monorepo with three distinct layers:

### 1. The Core (`packages/env-architect`)
The intelligence engine. It handles:
*   Parsing manifests.
*   Resolving dependency graphs.
*   Validating constraints.

### 2. The Tooling
*   **CLI (`apps/cli`)**: The primary interface to architect and deploy environments.
*   **VS Code Extension**: Real-time intelligence and validation.

### 3. The Platform
*   **Registry API**: Central hub for storing and distributing environment definitions.
*   **Web Portal**: Dashboard for managing team environments.

## Reporting issues

Found a bug? We'd love to know about it!

Please report all issues on the GitHub [issue tracker][issues].

[issues]: https://github.com/athexweb3/envArchitect/issues

## Contributing

See the **[Contributor Guide]** for a complete introduction to contributing to EnvArchitect.

## License

EnvArchitect is primarily distributed under the terms of both the MIT license
and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
