# Appendix

This is the appendix section of the test book.

## Additional Information

Some supplementary information that complements the main content.

### Code Examples

Here's a code example with proper language tagging:

```rust
fn appendix_example() {
    println!("This code is properly formatted");
}
```

But this one lacks a language tag:

```
def python_example():
    print("This should trigger MDBOOK001")
```

## Reference Links

[Working internal link](./intro.md) - this should be fine

[Broken reference link](./missing-appendix.md) - should trigger MDBOOK002

## Final Notes

This appendix provides additional context and examples for the test book. It includes both clean content and intentional violations to support comprehensive testing of the mdbook-lint tool.

The appendix serves as a realistic example of how supplementary content might be structured in a real mdBook project.