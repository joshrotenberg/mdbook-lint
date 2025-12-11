# Compatibility

mdbook-lint is designed to work seamlessly across different versions of mdBook and various environments.

## mdBook Version Support

mdbook-lint supports both mdBook 0.4.x and 0.5.x series.

### Supported Versions

| mdBook Version | Status | Notes |
|----------------|--------|-------|
| 0.4.40+ | Supported | Fully tested |
| 0.5.0 | Supported | JSON format changes handled automatically |
| 0.5.1+ | Supported | Latest recommended |

### Automatic Version Detection

When running as an mdBook preprocessor, mdbook-lint automatically detects the mdBook version and handles protocol differences transparently. You don't need to configure anything differently based on your mdBook version.

### mdBook 0.5.x Changes

mdBook 0.5.0 introduced breaking changes to the preprocessor JSON protocol:

- The `sections` field was renamed to `items` in book chapter structures
- The `__non_exhaustive` marker field was removed

mdbook-lint automatically normalizes these differences, so your configuration and usage remain the same regardless of which mdBook version you use.

## Platform Support

mdbook-lint provides prebuilt binaries for all major platforms:

| Platform | Architecture | Binary |
|----------|--------------|--------|
| Linux | x86_64 (glibc) | `mdbook-lint-linux-x86_64` |
| Linux | x86_64 (musl) | `mdbook-lint-linux-x86_64-musl` |
| macOS | Intel (x86_64) | `mdbook-lint-macos-x86_64` |
| macOS | Apple Silicon (aarch64) | `mdbook-lint-macos-aarch64` |
| Windows | x86_64 | `mdbook-lint-windows-x86_64.exe` |

### Minimum Rust Version

If building from source, mdbook-lint requires:

- **Rust Edition**: 2024
- **Minimum Supported Rust Version (MSRV)**: 1.85.0

## Configuration Compatibility

### markdownlint Compatibility

mdbook-lint aims for compatibility with [markdownlint](https://github.com/DavidAnson/markdownlint) rule definitions. Standard rules (MD001-MD060) follow the same semantics as markdownlint where applicable.

Configuration differences:

- mdbook-lint uses TOML configuration by default (`.mdbook-lint.toml`)
- YAML and JSON configuration formats are also supported
- Rule configuration options may have slightly different names

### CI Environment Support

mdbook-lint works in all major CI environments:

- GitHub Actions
- GitLab CI
- CircleCI
- Jenkins
- Azure Pipelines

See [CI vs Preprocessor](./ci-vs-preprocessor.md) for guidance on choosing the right integration approach.

## Continuous Compatibility Testing

mdbook-lint runs automated compatibility tests against multiple mdBook versions on a weekly schedule. These tests verify that the preprocessor integration works correctly across all supported mdBook versions.

You can view the test results in the [mdBook Compatibility workflow](https://github.com/joshrotenberg/mdbook-lint/actions/workflows/mdbook-compatibility.yml) on GitHub.

## Reporting Compatibility Issues

If you encounter compatibility issues with a specific mdBook version or platform, please [open an issue](https://github.com/joshrotenberg/mdbook-lint/issues) with:

1. Your mdBook version (`mdbook --version`)
2. Your mdbook-lint version (`mdbook-lint --version`)
3. Your operating system and architecture
4. The error message or unexpected behavior
5. A minimal example that reproduces the issue
