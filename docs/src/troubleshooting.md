# Troubleshooting Guide

This guide helps you resolve common issues with mdbook-lint.

## Table of Contents

- [Installation Issues](#installation-issues)
- [Configuration Problems](#configuration-problems)
- [Preprocessor Issues](#preprocessor-issues)
- [Performance Problems](#performance-problems)
- [Rule-Specific Issues](#rule-specific-issues)
- [CI/CD Problems](#cicd-problems)
- [Debugging Tips](#debugging-tips)

## Installation Issues

### Command Not Found

**Problem**: `mdbook-lint: command not found` after installation.

**Solutions**:

1. **Verify Cargo bin directory is in PATH**:

   ```bash
   echo $PATH | grep -q "$HOME/.cargo/bin" || echo "Not in PATH"
   export PATH="$HOME/.cargo/bin:$PATH"
   echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
   ```

2. **Check installation location**:

   ```bash
   find ~ -name mdbook-lint -type f 2>/dev/null
   ```

3. **Reinstall with verbose output**:

   ```bash

   cargo install mdbook-lint --force --verbose
   ```

### Version Conflicts

**Problem**: Different versions between CLI and preprocessor.

**Solution**:

```bash
# Check versions
mdbook-lint --version
cargo install --list | grep mdbook-lint

# Update to latest
cargo install mdbook-lint --force
```

### Build Failures During Installation

**Problem**: Compilation errors when installing from source.

**Solutions**:

1. **Update Rust toolchain**:

   ```bash
   rustup update stable
   rustup default stable
   ```

2. **Clear cargo cache**:

   ```bash
   cargo clean
   rm -rf ~/.cargo/registry/cache
   ```

3. **Install with specific version**:

   ```bash

   cargo install mdbook-lint --version 0.11.1
   ```

## Configuration Problems

### Configuration Not Loading

**Problem**: Settings in configuration files are ignored.

**Debug Steps**:

1. **Check configuration discovery**:

   ```bash

## Show which config file is being used

   mdbook-lint lint --debug src/ 2>&1 | grep -i config

   ```

2. **Validate configuration syntax**:
   ```bash
# For TOML

   cat .mdbook-lint.toml | python -m json.tool > /dev/null 2>&1 || echo "Invalid TOML"
   
# For JSON

   cat .mdbook-lint.json | jq . > /dev/null || echo "Invalid JSON"
   
# For YAML

   cat .mdbook-lint.yaml | python -c "import yaml, sys; yaml.safe_load(sys.stdin)" || echo "Invalid YAML"
   ```

3. **Test with explicit config**:

   ```bash

   mdbook-lint lint --config ./my-config.toml src/
   ```

## Rule Configuration Not Working

**Problem**: Rule-specific settings aren't applied.

**Example Working Configurations**:

```toml
# .mdbook-lint.toml
[rules.config]
# Correct: Use table syntax for rule config
MD013 = { line_length = 100, tables = false }
MD024 = { siblings_only = true }

# Wrong: Don't use this format
# MD013.line_length = 100  # This won't work
```

### Environment Variables Not Working

**Problem**: Environment variable overrides aren't applied.

**Correct Format**:

```bash
# Preprocessor settings
export MDBOOK_PREPROCESSOR__MDBOOK_LINT__FAIL_ON_WARNINGS=true
export MDBOOK_PREPROCESSOR__MDBOOK_LINT__DISABLED_RULES='["MD013","MD033"]'

# Note: Use JSON array format for lists
# Wrong: DISABLED_RULES="MD013,MD033"
# Right: DISABLED_RULES='["MD013","MD033"]'
```

## Preprocessor Issues

### Preprocessor Not Running

**Problem**: mdbook-lint doesn't execute during `mdbook build`.

**Comprehensive Check**:

```bash
#!/bin/bash
# Diagnostic script

echo "1. Checking mdbook-lint installation..."
which mdbook-lint || echo "ERROR: mdbook-lint not found in PATH"

echo "2. Checking book.toml..."
grep -A5 "preprocessor.lint" book.toml || echo "ERROR: Preprocessor not configured"

echo "3. Testing preprocessor directly..."
echo '{"root":"","config":{},"renderer":"html","mdbook_version":"0.4.0"}' | mdbook-lint preprocessor

echo "4. Checking mdbook version..."
mdbook --version

echo "5. Testing build with verbose output..."
mdbook build -v 2>&1 | grep -i mdbook-lint
```

### Preprocessor Crashes

**Problem**: Build fails with preprocessor errors.

**Debug Mode**:

```bash
# Enable debug logging
export RUST_LOG=mdbook_lint=debug
export RUST_BACKTRACE=1

# Run build
mdbook build 2> mdbook-lint-debug.log

# Check error details
grep ERROR mdbook-lint-debug.log
```

### Conflicts with Other Preprocessors

**Problem**: mdbook-lint conflicts with other preprocessors.

**Solution - Control execution order**:

```toml
# book.toml
[preprocessor.lint]
before = ["links"]  # Run before links preprocessor
after = ["index"]   # Run after index preprocessor

[preprocessor.other-processor]
after = ["mdbook-lint"]  # Ensure mdbook-lint runs first
```

## Performance Problems

### Slow Builds

**Problem**: mdbook build takes too long with linting enabled.

**Optimization Strategies**:

1. **Profile the slowdown**:

   ```bash

   time mdbook build --dest-dir book-without-lint
   
## With linting

   time mdbook build --dest-dir book-with-lint

   ```

2. **Disable expensive rules**:
   ```toml
   [preprocessor.lint]
# Line length and link checking are expensive

   disabled-rules = ["MD013", "MD053", "MDBOOK002"]
   ```

3. **Limit scope**:

   ```toml

   [preprocessor.lint]

## Only lint main content

   include = ["src/chapters/**/*.md"]
   exclude = ["src/appendix/**", "src/reference/**"]

   ```

4. **Use parallel processing** (if available):
   ```bash
   export RAYON_NUM_THREADS=4
   mdbook build
   ```

## Memory Issues

**Problem**: Out of memory errors on large books.

**Solutions**:

1. **Process files individually**:

   ```bash

## Instead of linting everything at once

   for file in src/**/*.md; do
     mdbook-lint lint "$file"
   done

   ```

2. **Increase memory limits**:
   ```bash
# Linux/macOS

   ulimit -v unlimited
   
# Or specify a limit

   ulimit -v 4194304  # 4GB
   ```

## Rule-Specific Issues

### False Positives

**Problem**: Rules flag valid content as violations.

**Solutions**:

1. **Disable rules inline**:

   ```markdown
   <!-- mdbook-lint-disable MD033 -->
   <div class="custom-element">
     This HTML is intentional
   </div>
   <!-- mdbook-lint-enable MD033 -->
   ```

2. **Configure rule parameters**:

   ```toml

   [rules.config]

## Allow specific HTML tags

   MD033 = { allowed_elements = ["div", "span", "details", "summary"] }

   ```

3. **Report false positives**:
   ```bash
# Create minimal reproduction

   echo "# Test\n<valid-html></valid-html>" > test.md
   mdbook-lint lint test.md
   
# Report issue with output

   ```

## Rule Conflicts

**Problem**: Different rules want opposite formatting.

**Example Resolution**:

```toml
# MD047 wants files to end with newline
# MD012 limits consecutive blank lines
# Resolution: Configure both appropriately
[rules.config]
MD047 = true  # Require final newline
MD012 = { maximum = 1 }  # But only one
```

## CI/CD Problems

### GitHub Actions Failures

**Problem**: CI passes locally but fails in GitHub Actions.

**Debug Workflow**:

```yaml
- name: Debug environment
  run: |
    echo "PATH: $PATH"
    which mdbook-lint || echo "mdbook-lint not found"
    mdbook-lint --version || echo "Version check failed"
    
- name: Debug configuration
  run: |
    cat book.toml
    ls -la .mdbook-lint.* 2>/dev/null || echo "No config files"
    
- name: Test with explicit verbosity
  run: |
    export RUST_LOG=debug
    mdbook build -v
```

### Docker Container Issues

**Problem**: mdbook-lint fails in Docker containers.

**Working Dockerfile**:

```dockerfile
FROM rust:1.70 AS builder

# Install mdbook and mdbook-lint
RUN cargo install mdbook mdbook-lint

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/mdbook* /usr/local/bin/

WORKDIR /book
CMD ["mdbook", "build"]
```

## Debugging Tips

### Enable Verbose Logging

```bash
# Maximum verbosity
export RUST_LOG=trace
export RUST_BACKTRACE=full

# Run with timing information
time mdbook-lint lint src/ --verbose
```

### Create Minimal Reproduction

```bash
#!/bin/bash
# Create minimal test case

mkdir mdbook-lint-test
cd mdbook-lint-test

# Create minimal book
cat > book.toml << EOF
[book]
title = "Test"
authors = ["Test"]

[preprocessor.lint]
fail-on-warnings = true
EOF

mkdir src
echo "# Test\n\nThis is a test." > src/SUMMARY.md
echo "# Chapter 1" > src/chapter_1.md

# Test
mdbook build -v
```

### Check Binary Dependencies

```bash
# Linux
ldd $(which mdbook-lint)

# macOS
otool -L $(which mdbook-lint)

# Check for missing libraries
mdbook-lint --version || echo $?
```

### Trace System Calls

```bash
# Linux
strace -e open,stat mdbook-lint lint src/ 2>&1 | grep -E "\.(toml|yaml|json)"

# macOS
dtruss -t open mdbook-lint lint src/ 2>&1 | grep -E "\.(toml|yaml|json)"
```

## Getting Help

If these solutions don't resolve your issue:

1. **Search existing issues**:

   ```bash
   gh issue list --repo joshrotenberg/mdbook-lint --search "your error"
   ```

2. **Create detailed bug report**:

   ```bash
   mdbook-lint --version > bug-report.txt
   echo "---" >> bug-report.txt
   mdbook --version >> bug-report.txt
   echo "---" >> bug-report.txt
   cat book.toml >> bug-report.txt
   echo "---" >> bug-report.txt
   mdbook build -v 2>&1 | tail -50 >> bug-report.txt
   ```

3. **Join discussions**:


- GitHub Issues: <https://github.com/joshrotenberg/mdbook-lint/issues>

- Discussions: <https://github.com/joshrotenberg/mdbook-lint/discussions>

## Common Error Messages

### "Failed to parse configuration"

**Cause**: Syntax error in configuration file.

**Fix**: Validate configuration syntax (see [Configuration Problems](#configuration-problems)).

### "Rule not found: XXXX"

**Cause**: Typo in rule ID or using removed rule.

**Fix**: Check available rules with `mdbook-lint rules`.

### "Preprocessor failed: Input/Output error"

**Cause**: mdbook-lint crashed or timed out.

**Fix**: Check system resources and enable debug logging.

### "No such file or directory"

**Cause**: Incorrect paths in configuration.

**Fix**: Use absolute paths or paths relative to book root:

```toml
[preprocessor.lint]
include = ["src/**/*.md"]  # Relative to book root
exclude = ["/tmp/**"]       # Absolute path
```
