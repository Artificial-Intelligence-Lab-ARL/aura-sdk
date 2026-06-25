# Releasing and Versioning Guide

Aura SDK follows standard Semantic Versioning (SemVer) and uses Cargo ecosystem tooling to manage releases cleanly.

## Versioning Rules

We adhere strictly to [SemVer 2.0.0](https://semver.org/):
- **Major (`X.0.0`)**: Incompatible API changes (e.g., changing signatures of public library entrypoints in `GenieEngine` or `AuraEngine`).
- **Minor (`0.Y.0`)**: Backwards-compatible new features (e.g., adding a new backend engine).
- **Patch (`0.0.Z`)**: Backwards-compatible bug fixes and internal refactors.

## Automated Versioning with `cargo-release`

We use the standard cargo plugin `cargo-release` to automate version bumping, tagging, and publishing.

### 1. Installation

To install `cargo-release` on your machine:

```powershell
cargo install cargo-release
```

### 2. Pre-release Checks

Before making a release, ensure all checks pass:

```powershell
# Check formatting
cargo fmt --all -- --check

# Check clippy warnings
cargo clippy --all-targets --all-features -- -D warnings

# Run all test suites
cargo test --all-features
```

### 3. Executing a Release

You can release a patch, minor, or major version. `cargo-release` will automatically:
1. Bump the version in `Cargo.toml`.
2. Commit the changes.
3. Tag the commit with the new version (e.g. `v0.2.1`).
4. Push the commit and the tag to `origin/main`.
5. Trigger the GitHub Actions CI/CD pipeline which will build release binaries, create a GitHub release, and publish the crate to [crates.io](https://crates.io).

To perform a dry run (simulate the release without making changes):

```powershell
cargo release patch --dry-run
```

To execute the actual release:

```powershell
cargo release patch --execute
```

For minor or major versions:

```powershell
cargo release minor --execute
cargo release major --execute
```

## Manual Releases

If you prefer to release manually:

1. Update the `version` field in `Cargo.toml`.
2. Document the changes under a new version heading in `CHANGELOG.md`.
3. Commit and tag:
   ```powershell
   git add Cargo.toml CHANGELOG.md
   git commit -m "chore: release version 0.2.0"
   git tag -a v0.2.0 -m "Release v0.2.0"
   git push origin main --tags
   ```
