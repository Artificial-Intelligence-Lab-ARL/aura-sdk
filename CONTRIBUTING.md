# Contributing to Aura SDK

First off, thank you for considering contributing to Aura SDK! It's people like you that make it a great tool.

## Code of Conduct

By participating in this project, you agree to abide by our Code of Conduct and standard open-source behaviors.

## Branching Strategy

To keep the repository clean and standardized, we follow a branching model similar to GitHub Flow:

- **`main`**: The primary branch where the code is always in a stable, buildable state. Direct commits to `main` are discouraged.
- **Feature Branches (`feat/...`)**: Used for adding new features or capabilities.
- **Bug Fix Branches (`fix/...`)**: Used for addressing issues, bug fixes, or minor corrections.
- **Refactoring Branches (`refactor/...`)**: Used for code restructure or cleanups without changes in behavior.
- **Documentation Branches (`docs/...`)**: Used for editing `.md` files or code docs.

### Pull Request Process

1. Create a branch from `main` using the appropriate prefix (`feat/`, `fix/`, etc.).
2. Implement your changes. Do not include comments in the source code unless absolutely necessary.
3. Verify your changes compile on Windows ARM64 (both native mode and ORT mode).
4. Run standard formatting and clippy checks:
   ```powershell
   cargo fmt --check
   cargo clippy --all-targets --all-features -- -D warnings
   ```
5. Submit a Pull Request targeting the `main` branch. A project maintainer will review it.

## Commit Message Guidelines

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

- **`feat: <description>`**: A new feature.
- **`fix: <description>`**: A bug fix.
- **`docs: <description>`**: Documentation changes.
- **`style: <description>`**: Formatting, white-space, etc. (no code change).
- **`refactor: <description>`**: A code change that neither fixes a bug nor adds a feature.
- **`perf: <description>`**: A code change that improves performance.
- **`test: <description>`**: Adding or correcting tests.
- **`chore: <description>`**: Build process, auxiliary tools, dependency updates, etc.

## Release Process and Versioning

We adhere to [Semantic Versioning (SemVer)](https://semver.org/). 

1. **Major Version (`X.0.0`)**: Backwards-incompatible API changes.
2. **Minor Version (`0.Y.0`)**: Backwards-compatible new features.
3. **Patch Version (`0.0.Z`)**: Backwards-compatible bug fixes.

Releases are published to crates.io and tagged on GitHub as `vX.Y.Z`. Every release must be accompanied by an update to the `CHANGELOG.md` file.
