# Automated Release and Changelog Generation

This guide explains how to use the automated versioning and changelog system configured for Aura SDK.

## Core Components

The automation relies on three standard tools:
1. **Conventional Commits**: A specification for writing commit messages that tools can easily parse.
2. **`git-cliff`**: An open-source tool that reads the git history, parses Conventional Commits, and generates a structured `CHANGELOG.md` file using `cliff.toml`.
3. **`cargo-release`**: A Cargo extension that manages version bumps, git tagging, commits, pushes, and triggers the CI/CD pipeline.

---

## 1. How It Works

We have integrated these tools using a configuration file called `release.toml`.

```
[Local Development] 
       │ 
       ▼ (Merge features to main branch using Conventional Commits)
[Run `cargo release <bump>`] 
       │ 
       ├──► 1. Bumps version in `Cargo.toml`
       ├──► 2. Runs `git-cliff` to parse history & write `CHANGELOG.md`
       ├──► 3. Commits changes & tags them as `vX.Y.Z`
       └──► 4. Pushes changes to origin
       │
[GitHub Actions CI/CD] 
       │
       ├──► 1. Runs formatting, clippy, and tests
       ├──► 2. Builds Windows ARM64 release binaries
       ├──► 3. Publishes release assets to GitHub Releases
       └──► 4. Publishes library package to crates.io
```

---

## 2. Standard Workflow

Follow these steps to develop and publish a release:

### Step A: Write Conventional Commit Messages
Ensure all your commits follow the [Conventional Commits](https://www.conventionalcommits.org/) format.

Examples:
- `feat: add support for local socket communication`
- `fix: resolve memory leak in GenieEngine drop handler`
- `perf: optimize token serialization processing speed`
- `docs: update release automation guides`

### Step B: Trigger the Release
When you are ready to publish a new version, run the release command from the `main` branch.

To simulate the release (dry run):
```powershell
cargo release patch --dry-run
```

To execute the release (this will bump the patch version, e.g., `0.2.0` -> `0.2.1`):
```powershell
cargo release patch --execute
```

For minor version changes (`0.2.0` -> `0.3.0`):
```powershell
cargo release minor --execute
```

---

## 3. Configuration Files

- **`cliff.toml`**: Customizes how `git-cliff` groups your commits (e.g. Features, Bug Fixes, Performance) and templates the changelog style.
- **`release.toml`**: Configures `cargo-release` behavior and binds the `git-cliff` pre-release hook so the changelog regenerates dynamically.
