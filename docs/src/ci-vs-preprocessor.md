# CI vs Preprocessor: Choosing Your Integration Strategy

This guide helps you choose between running mdbook-lint as an mdBook preprocessor or as a standalone CLI tool in CI, and explains why you typically want one approach but not both.

## TL;DR Recommendation

**For CI/CD pipelines: Use the standalone CLI.**
**For local development: Use the preprocessor (optional).**

The standalone CLI gives you more control, better error handling, and avoids configuration discovery issues that can occur in preprocessor mode.

## Quick Decision Guide

| Use Case | Recommended Approach | Why |
|----------|---------------------|-----|
| CI/CD pipelines | **Standalone CLI** | More control, fail fast, better error output |
| Local development with mdBook | **Preprocessor** | Automatic feedback during `mdbook serve` |
| Pure markdown documentation (no mdBook) | **Standalone CLI** | No mdBook dependency needed |
| Need SARIF/GitHub integration | **Standalone CLI** | Better tool integration options |
| Complex CI pipeline with multiple checks | **Standalone CLI** | More control over when/how linting runs |

## Integration Approaches

### Approach 1: mdBook Preprocessor (Best for Local Development)

**When to use:**

- You want automatic linting during `mdbook serve`
- You prefer configuration in `book.toml`
- You want immediate feedback while writing

**When NOT to use:**

- In CI/CD pipelines (use standalone CLI instead)
- When you need precise control over exit codes
- When you need detailed error output for debugging

**Setup in `book.toml`:**

```toml
[preprocessor.lint]
fail-on-warnings = false  # Set to true for strict mode
disabled-rules = ["MD013", "MD033"]

[MD007]
indent = 4
```

**In CI (GitHub Actions):**

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install mdBook and mdbook-lint

        run: |

          cargo install mdbook
          cargo install mdbook-lint
      - name: Build book (linting happens automatically)
        run: mdbook build
        env:
# Optional: Override settings for CI

          MDBOOK_PREPROCESSOR__MDBOOK_LINT__FAIL_ON_WARNINGS: true
```

**Advantages:**

- Linting happens automatically during `mdbook serve`
- Immediate feedback while writing documentation
- Works seamlessly with local mdBook workflow

**Disadvantages:**

- Configuration discovery can be tricky in CI environments
- Limited control over error handling and exit codes
- No SARIF output for GitHub Security tab
- Errors appear inline with mdBook build output
- Can't fail fast in CI (must start book build first)

### Approach 2: Standalone CLI (Recommended for CI/CD)

**When to use:**

- CI/CD pipelines (recommended for all CI use cases)
- You want to fail fast before other expensive operations
- You need clear, actionable error output
- You need SARIF output for GitHub Security integration
- You don't use mdBook (just markdown files)

**Setup with `.mdbook-lint.toml`:**

```toml
fail-on-warnings = true
disabled-rules = ["MD013", "MD033"]

[MD007]
indent = 4
```

**In CI (GitHub Actions):**

```yaml
jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
# Option A: Using GitHub Action

      - name: Lint Markdown
        uses: joshrotenberg/mdbook-lint-action@v1
        with:
          files: 'docs/**/*.md'
          format: sarif
          output-file: results.sarif
      
# Option B: Direct installation

      - name: Install and run mdbook-lint

        run: |

          cargo install mdbook-lint
          mdbook-lint lint docs/ --fail-on-warnings
      
# Optional: Upload SARIF results

      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: results.sarif
```

**Advantages:**

- Fails fast in CI pipeline before expensive build steps
- Clear, standalone error output for debugging
- SARIF output for GitHub Security tab
- Full control over when and how linting runs
- Can run in parallel with other checks
- Works with or without mdBook
- Supports smart CLI detection (e.g., `mdbook-lint docs/`)

**Disadvantages:**

- No automatic linting during local `mdbook serve`
- Requires explicit invocation in CI workflow
- Consider adding preprocessor for local development feedback

## Why Not Both

Running mdbook-lint both as a preprocessor AND standalone in CI is usually redundant and can cause problems:

### Problems with Running Both

1. **Duplicate Work**: The same files get linted twice, wasting CI time
2. **Configuration Drift**: Two places to maintain rules can lead to inconsistencies
3. **Confusing Failures**: Issues might be reported twice in different formats
4. **Maintenance Burden**: Updates need to be synchronized in multiple places

### Valid Exception: Different Rule Sets

The only scenario where using both makes sense is when you intentionally want different rules:

```yaml
# CI: Strict linting before build
- name: Strict lint check
  run: mdbook-lint lint docs/ --config .mdbook-lint.strict.toml

# Build: Lenient linting during build
- name: Build with lenient linting
  run: mdbook build  # Uses preprocessor with book.toml config
```

## Migration Strategies

### From Preprocessor to Standalone CI

If you're currently using the preprocessor but want to switch to standalone CI:

1. **Extract configuration** from `book.toml` to `.mdbook-lint.toml`
2. **Remove preprocessor** section from `book.toml`
3. **Update CI** to run mdbook-lint before mdbook build
4. **Document the change** for your team

### From Standalone to Preprocessor

If you're using standalone but want to switch to preprocessor:

1. **Add preprocessor** section to `book.toml`
2. **Copy configuration** from `.mdbook-lint.toml` to `book.toml`
3. **Remove standalone lint step** from CI
4. **Update documentation** for developers

## Recommended Configurations

### For CI/CD: Use Standalone CLI (Recommended)

```yaml
# .github/workflows/docs.yml
jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install mdbook-lint
        run: cargo install mdbook-lint
      - name: Lint documentation
        run: mdbook-lint docs/src/ --fail-on-warnings
```

### For Local Development: Add Preprocessor (Optional)

```toml
# book.toml - for local development feedback only
[preprocessor.lint]
fail-on-warnings = false
```

### Combined Setup (Best of Both Worlds)

Use standalone CLI in CI for control and reliability, with optional preprocessor for local development:

```yaml
# .github/workflows/docs.yml - CI uses standalone
- name: Lint documentation
  run: mdbook-lint docs/src/ --fail-on-warnings
- name: Build book
  run: mdbook build docs/
```

```toml
# book.toml - local development uses preprocessor (optional)
[preprocessor.lint]
fail-on-warnings = false
```

This approach gives you:

- Reliable CI with clear error output
- Fast feedback during local `mdbook serve`
- No duplicate configuration (use `.mdbook-lint.toml` for both)

## Common Pitfalls to Avoid

1. **Don't duplicate the same rules** in both preprocessor and standalone configs
2. **Don't run both in CI** unless you have a specific reason

3. **Don't use `|| true` to ignore failures** - fix the issues or disable specific rules

4. **Don't forget to document** which approach you're using for new contributors

## Summary

- **Use standalone CLI in CI**: Better control, clearer errors, fail-fast capability
- **Use preprocessor for local development**: Optional, provides feedback during `mdbook serve`
- **Avoid using both in CI**: Redundant and can cause confusion
- **Share configuration**: Use `.mdbook-lint.toml` which works for both modes
- **Be consistent**: Document your choice and stick with it across your project
