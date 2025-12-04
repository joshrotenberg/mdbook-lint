# Clean mdBook Test Document

This document follows all mdBook linting rules and should produce no violations.

## Proper Code Blocks

All code blocks have appropriate language tags:

```rust
# fn main() {
println!("Hello, Rust!");
# }
```

```bash
echo "Hello from bash"
```

```json
{
    "name": "test",
    "version": "1.0.0"
}
```

```toml
[package]
name = "example"
version = "0.1.0"
```

## Proper Heading Structure

Headings increment properly without skipping levels.

### Level 3 Heading

Content under level 3.

#### Level 4 Heading

Content under level 4.

## Proper Link Structure

All internal links point to valid files:

- [Link to self](mdbook_clean.md)
- [Link to violations test](mdbook_violations.md)

External links are fine:

- [Rust Documentation](https://doc.rust-lang.org/)
- [mdBook Guide](https://rust-lang.github.io/mdBook/)

## Consistent Lists

Unordered lists use consistent markers:

- First item
- Second item
- Third item

Ordered lists are properly formatted:

1. First step
2. Second step
3. Third step

## Proper Line Length

All lines in this document are kept under 80 characters to comply with
default line length rules.

## Proper Spacing

Block quotes have proper spacing around them:

> This is a properly formatted block quote with blank lines before and
> after it.

Code blocks also have proper spacing.

## Summary

This document demonstrates proper mdBook markdown formatting that should
pass all linting rules without violations.

