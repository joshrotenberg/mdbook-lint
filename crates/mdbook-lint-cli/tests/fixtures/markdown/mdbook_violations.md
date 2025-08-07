# mdBook Integration Test Fixtures

This file contains various markdown patterns to test mdBook-specific linting rules.

## MDBOOK001: Code blocks without language tags

Good code blocks:

```rust
fn main() {
    println!("Hello, Rust!");
}
```

```bash
echo "Hello from bash"
```

Bad code blocks (should trigger MDBOOK001):

```
fn missing_language() {
    println!("This should trigger MDBOOK001");
}
```

```
echo "Missing bash language tag"
```

## MDBOOK002: Internal link validation

Good internal links:
- [Valid relative link](./chapter1.md)
- [Link to summary](SUMMARY.md)

Bad internal links (should trigger MDBOOK002):
- [Broken link](./nonexistent.md)
- [Another broken link](../missing/file.md)

## Mixed standard and mdBook violations

### Heading level skip (MD001)

# Main Title

### Skipped Level 2 (should trigger MD001)

### MDBOOK004: Duplicate chapter titles test

This content would be used to test duplicate titles across multiple files.

## Standard rule violations for integration testing

This line is way too long and should trigger MD013 line length rule when configured with default settings of 80 characters per line.

```
This code block has no language tag and should trigger MDBOOK001
```

*   Inconsistent list marker (should trigger MD004)
-   Mixed with dash marker

> Block quote without blank lines around it
Should trigger MD022 if enabled.

## Additional test content

Some **bold text** and *italic text* for completeness.

1. Ordered list item
2. Another item

- Unordered list
- Another item

[External link](https://example.com) should not trigger MDBOOK002.