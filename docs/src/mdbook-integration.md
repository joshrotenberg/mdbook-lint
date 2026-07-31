# mdBook Integration

mdbook-lint can run as an mdBook preprocessor, so every chapter is checked as
part of `mdbook build` and `mdbook serve`. The preprocessor reports diagnostics
without changing chapter content.

If you are deciding between build-time checks and a separate CI lint step, see
[CI vs Preprocessor](./ci-vs-preprocessor.md).

## Installation

Install mdbook-lint where mdBook can find it on `PATH`:

```bash
cargo install mdbook-lint
```

Prebuilt binaries are also available from
[GitHub Releases](https://github.com/joshrotenberg/mdbook-lint/releases).

Verify the installation before configuring the book:

```bash
mdbook-lint --version
mdbook --version
```

## Basic setup

Add the preprocessor to `book.toml`:

```toml
[preprocessor.lint]
```

mdBook derives the `mdbook-lint` command from the `lint` preprocessor name. You
can make the command explicit if needed:

```toml
[preprocessor.lint]
command = "mdbook-lint"
```

The preprocessor now runs during normal mdBook commands:

```bash
mdbook build
mdbook serve
mdbook test
```

By default, warnings are reported without failing the build. Errors fail the
build.

## Configuration

There are two supported configuration sources for preprocessor mode:

1. A discovered mdbook-lint configuration file, preferably
   `.mdbook-lint.toml`.
2. Supported keys in `[preprocessor.lint]` in `book.toml`.

The external file is the place for rule-specific settings and the broader CLI
configuration surface:

```toml
# .mdbook-lint.toml
fail-on-warnings = false
disabled-rules = ["MD013", "MD033"]

[MD024]
siblings_only = true

[MD040]
language_optional = false
```

See [Configuration](./configuration.md) for all global settings and
rule-specific syntax.

### Supported `book.toml` keys

The `[preprocessor.lint]` table accepts these mdbook-lint settings:

| Setting | Type | Purpose |
|---------|------|---------|
| `fail-on-warnings` | boolean | Fail the mdBook build when warnings are found |
| `fail-on-errors` | boolean | Fail the mdBook build when errors are found |
| `enabled-rules` | array | Run only the listed rule IDs |
| `disabled-rules` | array | Skip the listed rule IDs |
| `enabled-categories` | array | Enable the listed rule categories |
| `disabled-categories` | array | Disable the listed rule categories |

For example:

```toml
[preprocessor.lint]
fail-on-warnings = true
disabled-rules = ["MD013", "MD041"]
disabled-categories = ["whitespace"]
```

Rule-specific tables and options such as `ignore-paths` belong in the external
configuration file. They are not read from `book.toml`.

### Configuration precedence

Preprocessor configuration is resolved in this order, with later supported
values overriding earlier values:

1. Built-in defaults.
2. The first configuration file discovered from the book root upward.
3. `[preprocessor.lint]` or `[preprocessor.mdbook-lint]` in `book.toml`.

The discovery order at each directory is:

1. `.mdbook-lint.toml`
2. `mdbook-lint.toml`
3. `.mdbook-lint.yaml`
4. `.mdbook-lint.yml`
5. `.mdbook-lint.json`

The CLI `--config` option applies to standalone commands such as
`mdbook-lint lint`; mdBook does not pass it to the preprocessor.

### Environment variables

mdbook-lint does not provide environment-variable configuration overrides.
Set build behavior in `book.toml`, use a discovered configuration file, or run
the standalone CLI with explicit flags in CI.

## Common workflows

### Strict CI and informational local builds

Keep the preprocessor informational for local builds:

```toml
[preprocessor.lint]
fail-on-warnings = false
```

Run a separate strict lint step in CI:

```bash
mdbook-lint lint --config .mdbook-lint.toml --fail-on-warnings src/
mdbook build
```

This avoids relying on an environment-variable override and gives CI direct
control over paths and output format.

### Progressive adoption

Start with a small allow-list and expand it over time:

```toml
[preprocessor.lint]
enabled-rules = ["MD001", "MD003", "MD009", "MD040", "MD047"]
```

Remove `enabled-rules` when the book is ready to use the default rule set.

### Rule-specific configuration

Keep rule behavior in `.mdbook-lint.toml`:

```toml
[MD003]
style = "atx"

[MD013]
line_length = 100
code_blocks = false

[MD033]
allowed_elements = ["details", "summary"]
```

Validate the file before building:

```bash
mdbook-lint check .mdbook-lint.toml
```

## GitHub workflows

The most predictable CI setup runs the CLI explicitly and then builds the book:

```yaml
name: Documentation

on:
  push:
  pull_request:

jobs:
  docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Install documentation tools
        run: |
          cargo install mdbook
          cargo install mdbook-lint

      - name: Validate lint configuration
        run: mdbook-lint check .mdbook-lint.toml

      - name: Lint documentation
        run: mdbook-lint lint --fail-on-warnings --output github src/

      - name: Build documentation
        run: mdbook build
```

Use `--config <FILE>` on the lint command if the configuration is not named for
automatic discovery.

## Limitations

- Preprocessor mode checks every chapter mdBook supplies. It does not support
  `include` or `exclude` chapter globs.
- Inline HTML comments cannot enable or disable rules for a portion of a file.
- Preprocessor diagnostics do not have selectable concise, detailed, or JSON
  formats. The standalone `lint` command supports `default`, `json`, and
  `github` output.
- mdbook-lint does not consume `RUST_LOG`; use `--verbose` with standalone CLI
  commands and `mdbook build -v` for mdBook diagnostics.

## Troubleshooting

If the preprocessor does not run, first check that both executables are on
`PATH` and that `book.toml` contains `[preprocessor.lint]`:

```bash
command -v mdbook
command -v mdbook-lint
mdbook build -v
```

For configuration problems, validate the external file and compare standalone
behavior:

```bash
mdbook-lint check .mdbook-lint.toml
mdbook-lint --verbose lint --config .mdbook-lint.toml src/
```

See the [Troubleshooting Guide](./troubleshooting.md) for more diagnostic steps.

## Next steps

- [Configuration](./configuration.md)
- [CLI Usage](./cli-usage.md)
- [CI vs Preprocessor](./ci-vs-preprocessor.md)
- [Troubleshooting](./troubleshooting.md)
