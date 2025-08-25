# MDBOOK008 - Invalid Rustdoc Include

**Severity**: Error  
**Category**: mdBook-specific  
**Auto-fix**: Not available

## Rule Description

This rule validates `{{#rustdoc_include}}` directives used to include Rust code from external files with proper rustdoc handling. It ensures correct syntax and that referenced files exist.

## Why This Rule Exists

Valid rustdoc includes are important because:
- Ensures Rust code examples compile and run correctly
- Preserves rustdoc annotations and attributes
- Maintains testable documentation
- Enables code reuse from actual Rust projects
- Prevents broken code examples in documentation

## Examples

### ❌ Incorrect (violates rule)

```markdown
<!-- File doesn't exist -->
{{#rustdoc_include ./missing.rs}}

<!-- Invalid syntax -->
{{#rustdoc_includes ./file.rs}}
{{#rust_doc_include ./file.rs}}

<!-- Invalid line range -->
{{#rustdoc_include ./main.rs:a:b}}
{{#rustdoc_include ./main.rs:-10}}
```

### ✅ Correct

```markdown
<!-- Include entire Rust file -->
{{#rustdoc_include ./examples/main.rs}}

<!-- Include specific lines -->
{{#rustdoc_include ./src/lib.rs:1:20}}
{{#rustdoc_include ./src/lib.rs:10:}}
{{#rustdoc_include ./src/lib.rs::15}}

<!-- Include with anchor -->
{{#rustdoc_include ./src/lib.rs:example_function}}
```

## Rustdoc Include vs Regular Include

### Key Differences

| Feature | `{{#include}}` | `{{#rustdoc_include}}` |
|---------|---------------|------------------------|
| Hidden lines (`# `) | Shown as-is | Hidden in output |
| Doc comments | Shown as-is | Processed correctly |
| Rust syntax highlighting | Manual | Automatic |
| Test annotations | Ignored | Preserved |

### Example Comparison

Source file `example.rs`:
```rust
# use std::collections::HashMap;
/// Creates a new cache
fn create_cache() -> HashMap<String, String> {
    # let mut cache = HashMap::new();
    # cache.insert("key".into(), "value".into());
    cache
}
```

With `{{#include ./example.rs}}`:
```rust
# use std::collections::HashMap;
/// Creates a new cache
fn create_cache() -> HashMap<String, String> {
    # let mut cache = HashMap::new();
    # cache.insert("key".into(), "value".into());
    cache
}
```

With `{{#rustdoc_include ./example.rs}}`:
```rust
/// Creates a new cache
fn create_cache() -> HashMap<String, String> {
    cache
}
```

## Configuration

```toml
[rules.MDBOOK008]
check_compilation = false  # Try to compile included code (default: false)
allow_external = false    # Allow includes outside project (default: false)
validate_anchors = true   # Check anchor existence (default: true)
```

## Common Issues and Solutions

### Issue: Hidden Lines Not Working
```markdown
<!-- Wrong: Using regular include -->
{{#include ./example.rs}}  <!-- Shows # lines -->

<!-- Correct: Using rustdoc_include -->
{{#rustdoc_include ./example.rs}}  <!-- Hides # lines -->
```

### Issue: Doc Tests Not Running
```rust
// example.rs
/// ```
/// assert_eq!(2 + 2, 4);
/// ```
fn documented_function() {}
```

```markdown
<!-- Use rustdoc_include to preserve doc tests -->
{{#rustdoc_include ./example.rs}}
```

### Issue: Anchor Mismatch
```rust
// ANCHOR: my_example
fn example() {
    // code
}
// ANCHOR_END: my_example
```

```markdown
<!-- Wrong anchor name -->
{{#rustdoc_include ./code.rs:myexample}}  ✗

<!-- Correct anchor name -->
{{#rustdoc_include ./code.rs:my_example}}  ✓
```

## Best Practices

1. **Use for Rust code**: Only use rustdoc_include for `.rs` files
2. **Hide setup code**: Use `#` prefix for boilerplate
3. **Include from src/**: Reference actual project code when possible
4. **Document anchors**: Explain what each anchor demonstrates
5. **Test included code**: Ensure examples compile and run

### Rust File Organization

```rust
// examples/complete_example.rs

// ANCHOR: imports
# use std::collections::HashMap;
# use std::error::Error;
// ANCHOR_END: imports

// ANCHOR: main_example
/// Main example function
pub fn example() -> Result<(), Box<dyn Error>> {
    # let data = HashMap::new();
    // Visible implementation
    println!("Processing data...");
    # Ok(())
}
// ANCHOR_END: main_example

# fn main() {
#     example().unwrap();
# }
```

### Including in Documentation

```markdown
## Complete Example

Here's how to use the library:

{{#rustdoc_include ./examples/complete_example.rs:main_example}}

The necessary imports are:

{{#rustdoc_include ./examples/complete_example.rs:imports}}
```

## When to Disable

Consider disabling this rule if:
- You're generating Rust files during the build
- You're migrating from regular includes gradually
- You use a custom preprocessor for Rust code
- Your examples are in a separate repository

### Disable in Config

```toml
# .mdbook-lint.toml
disabled_rules = ["MDBOOK008"]
```

### Disable Inline

```markdown
<!-- mdbook-lint-disable MDBOOK008 -->
{{#rustdoc_include ./generated/example.rs}}
<!-- mdbook-lint-enable MDBOOK008 -->
```

## Testing Included Code

### Verify Examples Compile

```bash
# Test all Rust code blocks
mdbook test

# Test specific chapter
mdbook test -c "Chapter Name"
```

### Extract and Test Separately

```bash
# Extract code examples
for file in examples/*.rs; do
    rustc --test "$file"
done
```

## Related Rules

- [MDBOOK007](./mdbook007.html) - Include directive validation
- [MDBOOK009](./mdbook009.html) - Playground directive syntax
- [MDBOOK012](./mdbook012.html) - Include line ranges

## References

- [mdBook - Including Rust Files](https://rust-lang.github.io/mdBook/format/markdown.html#including-rust-files)
- [Rustdoc - Documentation Tests](https://doc.rust-lang.org/rustdoc/documentation-tests.html)