# Release Process

This document describes the release process for mdbook-lint.

## Overview

mdbook-lint uses an automated release process based on conventional commits. When commits with specific patterns are pushed to main, the system automatically:

1. Detects the need for a release
2. Calculates the version bump (major/minor/patch)
3. Creates a release PR with updated versions and changelog
4. After PR merge, builds binaries and creates a GitHub release

## Automatic Release Triggers

The following commit types trigger releases:

- `feat:` - New features (triggers minor version bump)
- `fix:` - Bug fixes (triggers patch version bump)
- `perf:` - Performance improvements (triggers patch version bump)
- Breaking changes (with `!` or `BREAKING CHANGE:`) trigger major version bumps

## Manual Release Process

If you need to trigger a release manually:

1. Go to Actions → "Release Automation"
2. Click "Run workflow"
3. Select the main branch
4. Click "Run workflow"

## Release PR Process

When a release PR is created:

1. Review the changelog for accuracy
2. Verify version numbers are correct
3. Run a test build locally: `cargo build --release`
4. Merge the PR to trigger the actual release

## After PR Merge

The release workflow automatically:

1. Creates a git tag
2. Builds binaries for all platforms:
   - Linux (x86_64, aarch64)
   - macOS (x86_64, aarch64)
   - Windows (x86_64)
3. Creates a GitHub release with:
   - Generated changelog
   - Binary artifacts
   - Source code archives

## Version Management

The project uses Rust workspace versioning. All version updates happen in:

- `Cargo.toml` (workspace.package.version)
- Internal dependency references in crate Cargo.tomls

These are automatically synchronized by the release automation.

## Troubleshooting

### Release PR Not Created

Check that your commits follow conventional commit format:
```
feat: add new rule for X
fix: correct issue with Y
```

### Build Failures

Ensure all tests pass before merging the release PR:
```bash
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

### Version Conflicts

The automation handles version updates. If manual intervention is needed:

1. Update workspace version in root `Cargo.toml`
2. Update internal dependencies to match
3. Run `cargo build` to verify

## Configuration Files

- `.github/workflows/release-automation.yml` - Detects commits and creates PRs
- `.github/workflows/release.yml` - Builds binaries and creates releases
- `cliff.toml` - Changelog generation configuration

## Emergency Rollback

If a release needs to be rolled back:

1. Revert the merge commit on main
2. Delete the release and tag from GitHub
3. Fix the issue
4. Let automation create a new release PR

## Publishing to crates.io

Currently, mdbook-lint is not automatically published to crates.io. To publish manually:

```bash
cargo publish -p mdbook-lint-core
cargo publish -p mdbook-lint-rulesets
cargo publish -p mdbook-lint
```

Note: Publish in this order due to dependencies.