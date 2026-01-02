---
description: How to release a new version of EnvArchitect
---
# Release Workflow

## 1. Preparation
-   [ ] **Clean State**: Ensure `git status` is clean.
-   [ ] **Update Dependencies**: Run `cargo update`.
-   [ ] **Audit**:
    -   `cargo audit` (Security check)
    -   `cargo clippy -- -D warnings` (Lint check)
    -   `cargo test --workspace` (Unit tests)

## 2. Versioning
-   [ ] **Bumping**:
    -   Update version in `packages/env-architect/Cargo.toml`.
    -   Update version in `apps/cli/Cargo.toml`.
    -   Update version in `packages/sdks/rust/Cargo.toml`.
-   [ ] **Changelog**:
    -   Add entry to `CHANGELOG.md` (Keep a human-readable log).

## 3. Build Verification
-   [ ] **Release Build**: `cargo build --release`.
-   [ ] **Smoke Test**: Run `./target/release/env-architect --version`.

## 4. Git Operations
-   [ ] **Commit**: `git commit -am "chore: release vX.Y.Z"`
-   [ ] **Tag**: `git tag -s vX.Y.Z -m "Release vX.Y.Z"` (Signed tag preferred).
-   [ ] **Push**: `git push origin main --tags`.

## 5. Post-Release
-   [ ] **Verify CI**: Check GitHub Actions status.
-   [ ] **Announce**: Notify team/community.
