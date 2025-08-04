# Introduction

This introduction has some intentional violations for testing.

### Skipped heading level (should trigger MD001)

This line is way too long and should trigger MD013 line length rule when configured with default settings because it exceeds the typical 80 character limit.

```
fn missing_language() {
    println!("This code block has no language tag - should trigger MDBOOK001");
}
```

## Proper Section

Some content here.

```rust
fn good_code() {
    println!("This has a language tag");
}
```

[Broken internal link](./nonexistent.md)

> Block quote without proper spacing
Should trigger MD022.

Another code block without language:

```
echo "This also triggers MDBOOK001"
```
