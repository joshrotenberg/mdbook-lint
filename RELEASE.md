# Release Process

This document explains how releases are automated for mdbook-lint.

## Overview

The release process uses three tools working together:

| Tool | Purpose |
|------|---------|
| [release-plz](https://release-plz.dev/) | Creates release PRs, publishes to crates.io, creates git tags |
| [cargo-dist](https://opensource.axo.dev/cargo-dist/) | Builds release binaries for multiple platforms |
| [git-cliff](https://git-cliff.org/) | Generates changelogs from conventional commits |

## Automated Flow

```
Push to main
    │
    ▼
release-plz analyzes commits
    │
    ├─► No releasable changes → Done
    │
    └─► Releasable changes found
            │
            ▼
        Creates/updates release PR
        (version bumps + changelog)
            │
            ▼
        PR merged by maintainer
            │
            ▼
        release-plz publishes to crates.io
        (in correct dependency order)
            │
            ▼
        release-plz creates git tag (e.g., v0.14.1)
            │
            ▼
        Tag push triggers cargo-dist workflow
            │
            ▼
        cargo-dist builds binaries:
        - aarch64-apple-darwin (Mac ARM)
        - x86_64-apple-darwin (Mac Intel)
        - aarch64-unknown-linux-gnu (Linux ARM)
        - x86_64-unknown-linux-gnu (Linux x86)
        - x86_64-pc-windows-msvc (Windows)
            │
            ▼
        cargo-dist creates GitHub Release
        with all binaries attached
```

## Configuration Files

| File | Purpose |
|------|---------|
| `release-plz.toml` | release-plz configuration |
| `dist-workspace.toml` | cargo-dist configuration |
| `cliff.toml` | git-cliff changelog template |
| `.github/workflows/release-plz.yml` | release-plz workflow |
| `.github/workflows/release.yml` | cargo-dist workflow |

## Commit Convention

Releases are triggered by [Conventional Commits](https://www.conventionalcommits.org/):

| Prefix | Version Bump | Example |
|--------|--------------|---------|
| `feat:` | Minor (0.x.0) | `feat: add new rule MD061` |
| `fix:` | Patch (0.0.x) | `fix: correct anchor generation` |
| `feat!:` or `fix!:` | Minor (breaking) | `feat!: rename API method` |
| `docs:`, `test:`, `chore:` | No release | `docs: update README` |

## Secrets Required

| Secret | Purpose |
|--------|---------|
| `CARGO_REGISTRY_TOKEN` | Publishing to crates.io |
| `RELEASE_TOKEN` | PAT for tag pushes to trigger cargo-dist |
| `COMMITTER_TOKEN` | Updating Homebrew formula (optional) |

### Why RELEASE_TOKEN?

GitHub's `GITHUB_TOKEN` cannot trigger other workflows (security feature to prevent infinite loops). The `RELEASE_TOKEN` is a Personal Access Token that allows release-plz's tag push to trigger the cargo-dist workflow.

**Creating RELEASE_TOKEN:**
1. Go to https://github.com/settings/tokens?type=beta
2. Create fine-grained token with:
   - Repository access: Only this repository
   - Permissions: Contents (read/write), Pull requests (read/write)
3. Add as repository secret named `RELEASE_TOKEN`

## Manual Release (if needed)

If automation fails, you can release manually:

### 1. Publish to crates.io (in order)

```bash
# Must publish in dependency order
cargo publish -p mdbook-lint-core
cargo publish -p mdbook-lint-rulesets
cargo publish -p mdbook-lint
```

### 2. Create and push tag

```bash
VERSION=0.14.1
git tag v$VERSION
git push origin v$VERSION
```

### 3. Trigger cargo-dist (if tag push didn't trigger it)

```bash
# Delete and re-push tag from local machine
git push origin :refs/tags/v$VERSION
git push origin v$VERSION
```

Or if `workflow_dispatch` is enabled:
```bash
gh workflow run release.yml -f tag=v$VERSION
```

## Troubleshooting

### Release PR CI fails on "Release Check"

**Symptom:** `failed to select a version for the requirement mdbook-lint-core = "^X.Y.Z"`

**Cause:** Dry-run packaging can't resolve workspace dependencies that aren't published yet.

**Solution:** This is expected for version bumps. The actual release will work because release-plz publishes in the correct order.

### cargo-dist didn't trigger after release-plz

**Symptom:** Tag was created but no GitHub Release or binaries.

**Cause:** `RELEASE_TOKEN` secret might be missing or expired.

**Solution:**
1. Check/recreate the PAT
2. Manually trigger: delete and re-push the tag from local machine

### Changelog generation fails

**Symptom:** `Template render error: Variable not found`

**Cause:** git-cliff template using unavailable variables.

**Solution:** Check `cliff.toml` - use hardcoded values instead of `remote.github.*` variables.

### Wrong crate published first

**Symptom:** Publishing fails due to dependency version mismatch.

**Cause:** release-plz should handle order, but if manual publishing:

**Solution:** Always publish in order:
1. `mdbook-lint-core` (no internal deps)
2. `mdbook-lint-rulesets` (depends on core)
3. `mdbook-lint` (depends on both)

## Workspace Structure

```
mdbook-lint/
├── crates/
│   ├── mdbook-lint-core/      # Core library (published first)
│   ├── mdbook-lint-rulesets/  # Rule implementations (depends on core)
│   └── mdbook-lint-cli/       # CLI binary (depends on both)
├── release-plz.toml
├── dist-workspace.toml
└── cliff.toml
```

## Expected Artifacts

After a successful release, you should have:

### crates.io

Three crates published (check https://crates.io/crates/mdbook-lint):
- `mdbook-lint-core` - Core library
- `mdbook-lint-rulesets` - Rule implementations
- `mdbook-lint` - CLI binary

### GitHub Release

A GitHub Release (e.g., https://github.com/joshrotenberg/mdbook-lint/releases/tag/v0.14.1) with:

| Artifact | Platform |
|----------|----------|
| `mdbook-lint-aarch64-apple-darwin.tar.xz` | macOS ARM (Apple Silicon) |
| `mdbook-lint-x86_64-apple-darwin.tar.xz` | macOS Intel |
| `mdbook-lint-aarch64-unknown-linux-gnu.tar.xz` | Linux ARM64 |
| `mdbook-lint-x86_64-unknown-linux-gnu.tar.xz` | Linux x86_64 |
| `mdbook-lint-x86_64-pc-windows-msvc.zip` | Windows x86_64 |
| `dist-manifest.json` | cargo-dist manifest |
| `sha256.sum` | Checksums for all artifacts |
| `source.tar.gz` | Source archive |

Each binary archive also has a corresponding `.sha256` checksum file.

### Git Tag

A version tag matching the release (e.g., `v0.14.1`):
```bash
git tag -l "v*" | tail -1
```

## Checking Release Status

```bash
# Recent releases
gh release list --limit 5

# Check if tag exists
git tag -l "v*" | tail -5

# Check crates.io versions
cargo search mdbook-lint

# Check workflow runs
gh run list --workflow=release.yml --limit 5
gh run list --workflow=release-plz.yml --limit 5
```
