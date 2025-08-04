# mdBook Integration

mdbook-lint integrates seamlessly with mdBook as a preprocessor, automatically checking your markdown files during the build process.

## Setup

Add mdbook-lint to your `book.toml`:

```toml
[preprocessor.mdbook-lint]
```

That's it! mdbook-lint will now run automatically when you build your book.

## Configuration

You can configure the preprocessor behavior in your `book.toml`:

```toml
[preprocessor.mdbook-lint]
# Fail the build if linting issues are found
fail-on-warnings = true

# Disable specific rules
disabled-rules = ["MD013", "MD033"]

# Only run on specific chapters (optional)
# include = ["src/chapter1.md", "src/chapter2.md"]

# Exclude specific files (optional)
# exclude = ["src/draft.md"]
```

## Build Integration

When you run `mdbook build`, mdbook-lint will:

1. Check all markdown files in your book
2. Report any linting issues
3. Optionally fail the build if issues are found
4. Continue with normal mdBook processing

```bash
# Build with linting
mdbook build

# Watch mode also includes linting
mdbook serve
```

## CI/CD Integration

mdbook-lint works great in continuous integration:

```yaml
# .github/workflows/docs.yml
name: Documentation

on: [push, pull_request]

jobs:
  docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install mdbook-lint
        run: cargo install mdbook-lint
      - name: Build documentation
        run: mdbook build
      - name: Deploy to GitHub Pages
        if: github.ref == 'refs/heads/main'
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./book
```

## mdBook-Specific Rules

mdbook-lint includes special rules designed for mdBook projects:

- **MDBOOK001**: Check for proper SUMMARY.md structure
- **MDBOOK002**: Validate internal link references
- **MDBOOK003**: Check for missing files referenced in SUMMARY.md
- **MDBOOK004**: Validate mdBook-specific syntax

## Troubleshooting

### Preprocessor Not Running

If mdbook-lint isn't running during builds:

1. Ensure mdbook-lint is installed and in your PATH
2. Check that `[preprocessor.mdbook-lint]` is in your `book.toml`
3. Try running with verbose output: `mdbook build -v`

### Configuration Not Applied

Configuration precedence for the preprocessor:

1. Built-in defaults
2. `.mdbook-lint.toml` file
3. `book.toml` preprocessor configuration
4. Command-line arguments (when running CLI directly)

## Next Steps

- Learn about [Rules Reference](./rules-reference.md)
- See [Configuration Reference](./configuration-reference.md) for all options
- Check out [Contributing](./contributing.md) to help improve mdBook integration