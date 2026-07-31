# Troubleshooting Guide

This guide covers common installation, configuration, preprocessor, rule, and
CI problems. Commands and configuration examples match the current
mdbook-lint CLI.

## Installation

### Command Not Found

Confirm whether the executable is on `PATH`:

```bash
command -v mdbook-lint
mdbook-lint --version
```

For a Cargo installation, the executable normally lives in
`~/.cargo/bin`:

```bash
cargo install mdbook-lint --force
export PATH="$HOME/.cargo/bin:$PATH"
```

If mdBook runs in CI or a container, install mdbook-lint in that environment as
well. Installing it on the host does not make it available inside a container.

### Installation from Source Fails

Update the stable Rust toolchain and retry with locked dependencies:

```bash
rustup update stable
cargo install mdbook-lint --locked --force
```

Include the Rust version and the complete Cargo error when reporting a build
failure:

```bash
rustc --version
cargo --version
```

## Configuration

### Configuration Is Not Loading

Validate the file directly:

```bash
mdbook-lint check .mdbook-lint.toml
```

Then run linting with an explicit path and verbose status output:

```bash
mdbook-lint --verbose lint --config .mdbook-lint.toml src/
```

Without `--config`, mdbook-lint searches the current directory and its parents.
At each directory it checks these names in order:

1. `.mdbook-lint.toml`
2. `mdbook-lint.toml`
3. `.mdbook-lint.yaml`
4. `.mdbook-lint.yml`
5. `.mdbook-lint.json`

The verbose command prints the selected configuration path. A similarly named
file outside the search path is not loaded.

### Rule Configuration Is Ignored

Rule-specific configuration uses a top-level table named after the rule:

```toml
[MD013]
line_length = 100
code_blocks = false

[MD024]
siblings_only = true
```

Do not nest rule configuration under `[core]`, `[rules.config]`, or
`[preprocessor.lint.rules]`. Those tables are not part of the configuration
schema.

Start from the generated reference when you are unsure which options a rule
accepts:

```bash
mdbook-lint init --include-all --output reference.toml
mdbook-lint rules --detailed
```

### Configuration in `book.toml` Is Ignored

The mdBook preprocessor reads only these settings from `[preprocessor.lint]`:

- `fail-on-warnings`
- `fail-on-errors`
- `enabled-rules`
- `disabled-rules`
- `enabled-categories`
- `disabled-categories`

For rule-specific settings and other global options, create a discovered
`.mdbook-lint.toml` file in the book root or a parent directory.

For example:

```toml
# book.toml
[preprocessor.lint]
fail-on-warnings = true
disabled-rules = ["MD041"]
```

```toml
# .mdbook-lint.toml
[MD013]
line_length = 100
```

### Environment Variables Have No Effect

mdbook-lint does not implement environment-variable configuration overrides.
Names such as `MDBOOK_PREPROCESSOR__...`, `MDBOOK_LINT_CONFIG`, and `RUST_LOG`
are not read by the application.

Use one of the supported mechanisms instead:

- a discovered configuration file;
- supported `[preprocessor.lint]` keys in `book.toml`;
- `--config`, `--fail-on-warnings`, `--enable`, or `--disable` with the
  standalone CLI.

## Preprocessor Issues

### The Preprocessor Does Not Run

Check the executables and the mdBook configuration:

```bash
command -v mdbook
command -v mdbook-lint
grep -n "preprocessor.lint" book.toml
mdbook build -v
```

A minimal `book.toml` entry is:

```toml
[preprocessor.lint]
```

If command discovery is unusual in your environment, set it explicitly:

```toml
[preprocessor.lint]
command = "mdbook-lint"
```

### The Build Fails Unexpectedly

Warnings fail the build only when `fail-on-warnings = true`. Errors fail by
default. Run the linter directly to see the same diagnostics without mdBook's
output around them:

```bash
mdbook-lint lint src/
```

To check whether build policy is the cause, inspect `book.toml` and the
discovered configuration file for:

```toml
fail-on-warnings = true
fail-on-errors = true
```

Set `RUST_BACKTRACE=1` only when diagnosing a panic. It does not enable normal
application logging.

### Another Preprocessor Appears to Conflict

mdbook-lint does not modify chapter content, so content-order conflicts are
unusual. Temporarily remove other preprocessor tables from a copy of
`book.toml`, rebuild, and add them back one at a time to identify the source.

Do not rely on mdbook-lint-specific `before` or `after` keys; mdbook-lint does
not read or enforce them.

## Rule Behavior

### A Rule Reports a False Positive

First reproduce the result with only that rule enabled:

```bash
mdbook-lint lint --enable MD033 path/to/file.md
```

If the rule has supported options, configure its top-level table:

```toml
[MD033]
allowed_elements = ["details", "summary"]
```

Otherwise disable it globally or for that command:

```toml
disabled-rules = ["MD033"]
```

```bash
mdbook-lint lint --disable MD033 src/
```

Inline `mdbook-lint-disable` HTML comments are not implemented. If a rule needs
per-file or inline suppression, open a feature request separately from the
false-positive report.

When reporting a bug, include the smallest input that reproduces it, the rule
ID, the exact command, and `mdbook-lint --version`.

### Rules Appear to Conflict

Run each rule independently to identify which diagnostics and fixes overlap:

```bash
mdbook-lint lint --enable MD018 path/to/file.md
mdbook-lint lint --enable MD020 path/to/file.md
```

Preview automatic fixes before applying them:

```bash
mdbook-lint lint --fix --dry-run path/to/file.md
```

If two automatic fixes conflict, report both rule IDs and the original input.

## Performance

Compare build time with and without the preprocessor enabled in a temporary
copy of `book.toml`:

```bash
time mdbook build
```

Then narrow the rule set rather than adding unsupported chapter globs:

```toml
[preprocessor.lint]
enabled-rules = ["MD001", "MD003", "MD009", "MD040", "MD047"]
```

Preprocessor mode checks the chapters supplied by mdBook and does not implement
`include` or `exclude` patterns. If path-level filtering is required, run the
standalone CLI on explicit paths and use `ignore-paths` in
`.mdbook-lint.toml`.

## CI

### CI Differs from Local Results

Print tool versions and validate the same configuration used locally:

```bash
mdbook --version
mdbook-lint --version
mdbook-lint check .mdbook-lint.toml
mdbook-lint lint --config .mdbook-lint.toml --fail-on-warnings src/
```

Pinning versions in CI avoids changes caused by installing different releases
on different runs.

For GitHub Actions annotations, use the standalone output format:

```bash
mdbook-lint lint --output github --fail-on-warnings src/
```

Preprocessor output does not have a configurable JSON or GitHub format.

## Useful Diagnostics

Use the CLI's supported status and output options:

```bash
mdbook-lint --verbose lint src/
mdbook-lint lint --output json src/
mdbook-lint lint --color never src/
mdbook build -v
```

List rules and inspect configuration separately:

```bash
mdbook-lint rules --detailed
mdbook-lint check .mdbook-lint.toml
```

## Getting Help

Search or open an issue at
<https://github.com/joshrotenberg/mdbook-lint/issues>. Include:

- `mdbook-lint --version`;
- `mdbook --version` when preprocessor mode is involved;
- the relevant configuration with secrets removed;
- the smallest input that reproduces the problem;
- the exact command and complete diagnostic output.

For configuration syntax, also see [Configuration](./configuration.md). For
preprocessor setup, see [mdBook Integration](./mdbook-integration.md).
