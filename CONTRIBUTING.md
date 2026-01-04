# Contributing to EnvArchitect

Thank you for your interest in contributing to EnvArchitect! We welcome contributions from the community to help make this tool better.

## getting Started

1.  **Fork the repository** on GitHub.
2.  **Clone your fork** locally:
    ```bash
    git clone https://github.com/your-username/envArchitect.git
    cd envArchitect
    ```
3.  **Install dependencies**:
    Ensure you have Rust installed via [rustup](https://rustup.rs/).
    ```bash
    cargo build
    ```

## Development Workflow

### Building
To build the project in debug mode:
```bash
cargo build
```

### Testing
Run the test suite to ensure everything is working:
```bash
cargo test
```

### Code Style
We follow the standard Rust coding conventions. Please ensure your code is formatted before submitting:
```bash
cargo fmt
```
We also use `clippy` for linting:
```bash
cargo clippy
```

## Submitting Pull Requests

1.  Create a new branch for your feature or fix: `git checkout -b feature/my-feature`.
2.  Commit your changes with clear, descriptive messages (we follow [Conventional Commits](https://www.conventionalcommits.org/)).
3.  Push your branch to your fork.
4.  Open a Pull Request against the `main` branch of `athexweb3/envArchitect`.
5.  Wait for review and address any feedback.

## License
By contributing, you agree that your contributions will be licensed under the project's dual license (MIT OR Apache-2.0).
