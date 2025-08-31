# CI vs Preprocessor: Choosing Your Integration Strategy

This guide helps you choose between running mdbook-lint as an mdBook preprocessor or as a standalone tool in CI, and explains why you typically want one approach but not both.

## Quick Decision Guide

| Use Case | Recommended Approach | Why |
|----------|---------------------|-----|
| mdBook project with regular builds | **Preprocessor** | Automatic linting on every build |
| Pure markdown documentation (no mdBook) | **Standalone CI** | No mdBook dependency needed |
| Want fastest CI builds | **Standalone CI** | Can fail fast before building |
| Need SARIF/GitHub integration | **Standalone CI** | Better tool integration options |
| Want immediate feedback during development | **Preprocessor** | Catches issues during `mdbook serve` |
| Complex CI pipeline with multiple checks | **Standalone CI** | More control over when/how linting runs |

## Integration Approaches

### Approach 1: mdBook Preprocessor (Recommended for mdBook Projects)

**When to use:**
- You have an mdBook project
- You want linting integrated into your normal workflow
- You want consistent behavior between local development and CI
- You prefer configuration in `book.toml`

**Setup in `book.toml`:**
```toml
[preprocessor.mdbook-lint]
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
- ✅ Single source of truth for configuration
- ✅ Linting happens automatically during builds
- ✅ Works with `mdbook serve` for live feedback
- ✅ No duplicate configuration needed

**Disadvantages:**
- ❌ Can't fail fast in CI (must start book build first)
- ❌ Limited output format options
- ❌ No SARIF output for GitHub Security tab

### Approach 2: Standalone CI Tool

**When to use:**
- You don't use mdBook (just markdown files)
- You want to fail fast in CI before other expensive operations
- You need SARIF output for GitHub Security integration
- You want different linting rules in different CI contexts

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
- ✅ Fails fast in CI pipeline
- ✅ SARIF output for GitHub Security tab
- ✅ More control over when linting happens
- ✅ Can run in parallel with other checks
- ✅ Works without mdBook

**Disadvantages:**
- ❌ No automatic linting during local `mdbook serve`
- ❌ Need to maintain separate configuration
- ❌ Developers might forget to run locally

## Why Not Both?

Running mdbook-lint both as a preprocessor AND standalone in CI is usually redundant and can cause problems:

### Problems with Running Both:

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

### For mdBook Projects: Use Preprocessor

```toml
# book.toml
[preprocessor.mdbook-lint]
fail-on-warnings = false  # true in CI via env var

# .github/workflows/docs.yml
env:
  MDBOOK_PREPROCESSOR__MDBOOK_LINT__FAIL_ON_WARNINGS: true
```

### For Non-mdBook Projects: Use Standalone

```yaml
# .github/workflows/lint.yml
- uses: joshrotenberg/mdbook-lint-action@v1
  with:
    files: '**/*.md'
    fail-on-warnings: true
```

### For Maximum Flexibility: Standalone with Optional Preprocessor

```toml
# book.toml - minimal config for local development
[preprocessor.mdbook-lint]
fail-on-warnings = false

# CI runs standalone for proper control
# Developers get feedback during mdbook serve
```

## Common Pitfalls to Avoid

1. **Don't duplicate the same rules** in both preprocessor and standalone configs
2. **Don't run both in CI** unless you have a specific reason
3. **Don't use `|| true` to ignore failures** - fix the issues or disable specific rules
4. **Don't forget to document** which approach you're using for new contributors

## Summary

- **Use preprocessor**: When you have an mdBook project and want integrated linting
- **Use standalone**: When you need CI flexibility or don't use mdBook
- **Avoid using both**: Unless you specifically need different rule sets
- **Be consistent**: Document your choice and stick with it across your project