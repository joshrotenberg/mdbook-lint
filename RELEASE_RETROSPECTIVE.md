# Release Process Retrospective

## Executive Summary

The mdbook-lint project's release automation broke down due to fundamental incompatibilities between popular release tools and Rust workspace structures. After multiple attempts with different tools, we've settled on a custom GitHub Actions workflow with git-cliff for changelog generation.

## Timeline of Failures

### 1. Initial State: release-please (Failed)
**Problem**: "value at path package.version is not tagged" errors
**Root Cause**: release-please couldn't properly handle Rust workspaces with multiple crates
**Attempts**:
- Updated .release-please-manifest.json
- Created release-please-config.json with cargo-workspace plugin
- Manually created missing tags
- Synchronized versions across workspace

**Why it failed**: release-please's Rust support is primarily designed for single-crate projects. The cargo-workspace plugin exists but has poor documentation and unclear behavior with workspace.package inheritance.

### 2. Second Attempt: release-plz (Failed)
**Problem**: "these packages are not published to crates.io" errors
**Root Cause**: release-plz assumes all workspace members should be published to crates.io
**Attempts**:
- Added `publish = false` to internal crates
- Tried to configure release-plz to skip registry checks

**Why it failed**: release-plz is opinionated about crates.io publishing. Even with `publish = false`, it still validates against the registry, creating a chicken-and-egg problem for new projects.

### 3. Third Attempt: simple-release workflow (Partial Success)
**Problem**: Limited functionality, no changelog generation
**What worked**: Basic PR creation for releases
**What didn't**: No automatic version bumping, no changelog, manual process

### 4. Current Solution: Custom Automation
**Components**:
- release-automation.yml: Detects conventional commits, bumps versions, creates PRs
- release.yml: Builds binaries and creates GitHub releases
- git-cliff: Professional changelog generation

**Remaining Issues** (now fixed):
- Version dependency mismatches between workspace members
- Missing GitHub labels causing non-critical failures

## Root Causes Analysis

### 1. Tool Assumptions vs Reality

**Release Tools Assume**:
- Single package repositories or simple workspaces
- All packages published to registries
- Standard versioning without workspace inheritance
- Simple dependency relationships

**mdbook-lint Reality**:
- Complex workspace with 3 interdependent crates
- Internal crates not meant for crates.io
- Workspace.package version inheritance
- Circular development dependencies for testing

### 2. Documentation Gaps

- release-please's Rust workspace documentation is minimal
- release-plz doesn't clearly document the registry requirement
- No clear "best practices" guide for Rust workspace releases

### 3. Version Management Complexity

The workspace structure requires synchronized version updates across:
- workspace.package.version in root Cargo.toml
- Internal dependency versions in each crate
- Git tags
- GitHub releases

## Research: How Other Projects Handle This

### 1. tokio
- Uses custom release scripts in `.github/workflows/`
- Maintains a `tokio-release` tool specifically for their needs
- Manual changelog maintenance
- Lesson: Large projects often need custom tooling

### 2. rust-analyzer
- Uses `xtask` pattern with custom Rust code for releases
- Generates changelogs from PR titles
- Semi-automated process with human oversight
- Lesson: Automation + human review is common

### 3. rustfmt/clippy (rust-lang projects)
- Integrated into rust-lang/rust's release process
- Uses rustc's versioning
- Lesson: Not applicable for independent projects

### 4. cargo-edit
- Uses release-plz successfully
- All crates published to crates.io
- Simpler workspace structure
- Lesson: release-plz works when all crates are published

### 5. clap
- Custom Python scripts for release automation
- Detailed RELEASE.md checklist
- Uses cargo-release for version bumping
- Lesson: Hybrid approach with multiple tools

### 6. serde
- Manual release process
- Minimal automation
- Focus on stability over frequent releases
- Lesson: Sometimes simple is better

## Recommendations

### Immediate Actions
1. Fix version dependency mismatches (completed)
2. Document the release process in RELEASE.md
3. Add pre-release testing workflow

### Long-term Strategy

**Option 1: Enhance Current Custom Solution**
- Pros: Full control, tailored to our needs
- Cons: Maintenance burden, potential for bugs
- Enhancement ideas:
  - Add automatic crates.io publishing
  - Implement pre-release testing
  - Add rollback capabilities

**Option 2: cargo-release + Custom Workflow**
- Pros: Mature tool for version management, combine with our workflows
- Cons: Another tool to learn and configure
- Implementation: Use cargo-release for version bumping only

**Option 3: xtask Pattern**
- Pros: Rust-native, testable, version controlled
- Cons: More code to maintain
- Implementation: Create `xtask/` crate with release commands

## Lessons Learned

1. **Start simple**: Complex tools often don't fit complex projects
2. **Version synchronization is critical**: Must update all references atomically
3. **Test releases in a separate repository**: Avoid polluting main repo with failed attempts
4. **Document everything**: Future maintainers need to understand the process
5. **Consider the ecosystem**: Tools designed for crates.io don't work well for internal workspaces

## Current Working Solution

Our custom GitHub Actions workflow now:
- Detects conventional commits
- Calculates version bumps (major/minor/patch)
- Updates all version references correctly
- Generates professional changelogs with git-cliff
- Creates release PRs automatically
- Builds binaries for all platforms
- Creates GitHub releases with artifacts

## Next Steps

1. Monitor the current solution for stability
2. Add integration tests for the release process
3. Consider adding cargo-release for additional validation
4. Document the process for contributors