# MDBOOK009 - Invalid Playground Configuration

**Severity**: Warning  
**Category**: mdBook-specific  
**Auto-fix**: Not available

## Rule Description

This rule validates `{{#playground}}` directives and playground configuration in code blocks. It ensures proper syntax for making code examples editable and runnable in the Rust Playground.

## Why This Rule Exists

Valid playground configuration is important because:

- Enables interactive code examples for readers
- Ensures code can run in the Rust Playground
- Provides hands-on learning experiences
- Validates that examples are executable
- Maintains consistent playground behavior

## Examples

### ❌ Incorrect (violates rule)

````markdown
<!-- Invalid directive syntax -->
\{{#playgrounds ./file.rs}}
\{{playground ./file.rs}}

<!-- Invalid playground attributes -->
```rust,playground
fn main() {
    println!("Hello");
}
```

<!-- Missing file -->
\{{#playground ./missing.rs}}
````

### ✅ Correct

````markdown
<!-- Include file as playground -->
\{{#playground ./examples/hello.rs}}

<!-- Inline editable code -->
```rust,editable
fn main() {
    println!("Hello, world!");
}
```

<!-- Non-editable but runnable -->
```rust,no_run
fn main() {
    // This compiles but won't run
}
```

<!-- Editable with hidden lines -->
```rust,editable
# fn hidden_setup() {}
fn main() {
    println!("Visible code");
}
```
````

## Playground Attributes

### Code Block Attributes

| Attribute | Description |
|-----------|-------------|
| `editable` | Makes code block editable in browser |
| `no_run` | Code compiles but doesn't run |
| `compile_fail` | Example that should fail compilation |
| `ignore` | Code block is not tested |
| `should_panic` | Code should panic when run |

### Examples with Attributes

````markdown
<!-- Editable example -->
```rust,editable
fn main() {
    let x = 5;
    println!("x = {}", x);
}
```

<!-- Example that should fail -->
```rust,compile_fail
fn main() {
    let x: i32 = "not a number";
}
```

<!-- Example that panics -->
```rust,should_panic
fn main() {
    panic!("This is expected!");
}
```
````

## Configuration

```toml
# book.toml
[output.html.playground]
editable = true         # Make code blocks editable by default
copyable = true         # Add copy button to code blocks
copy-js = true          # Include JavaScript for copy functionality
line-numbers = false    # Show line numbers in code blocks

[MDBOOK009]
require_editable = false  # Require editable attribute (default: false)
validate_compilation = false  # Check if code compiles (default: false)
```

## Common Issues and Solutions

### Issue: Playground Not Enabled

```toml
# book.toml - Enable playground
[output.html.playground]
editable = true
```

### Issue: Code Doesn't Compile

````markdown
<!-- Missing main function -->
```rust,editable
println!("Hello");  // Error: not in a function
```

<!-- Fixed -->
```rust,editable
fn main() {
    println!("Hello");
}
```
````

### Issue: Hidden Lines Shown

````markdown
<!-- Wrong: Hidden lines visible in playground -->
```rust,editable
# use std::io;
# fn setup() {}
fn main() {
    // User code
}
```

<!-- Note: Hidden lines work but are still editable -->
````

## Best Practices

1. **Provide complete examples**: Include all necessary code
2. **Use hidden lines sparingly**: Only for necessary boilerplate
3. **Test in playground**: Verify examples work in actual playground
4. **Add context**: Explain what users should try changing
5. **Keep examples focused**: One concept per playground

### Interactive Example Template

````markdown
## Try It Yourself

Modify the code below to experiment with different values:

```rust,editable
fn calculate_area(width: f64, height: f64) -> f64 {
    width * height
}

fn main() {
    // Try changing these values!
    let width = 10.0;
    let height = 5.0;
    
    let area = calculate_area(width, height);
    println!("Area: {} square units", area);
    
    // Challenge: Add a perimeter calculation
}
```

**Suggestions to try:**
- Change the dimensions
- Add a perimeter function
- Handle negative values
````

## Advanced Usage

### Custom Playground URL

```toml
# book.toml
[output.html.playground]
editable = true
site = "https://play.rust-lang.org"
```

### Playground with Dependencies

````markdown
<!-- Note: Dependencies must be available in playground -->
```rust,editable
// Available in playground: rand, regex, lazy_static, etc.
use rand::Rng;

fn main() {
    let mut rng = rand::thread_rng();
    let n: u32 = rng.gen_range(0..10);
    println!("Random number: {}", n);
}
```
````

## When to Disable

Consider disabling this rule if:

- Your book doesn't use playground features
- You're targeting offline readers
- You have custom code execution environment
- Your examples require local dependencies

### Disable in Config

```toml
# .mdbook-lint.toml
disabled_rules = ["MDBOOK009"]
```

### Disable Inline

````markdown
<!-- mdbook-lint-disable MDBOOK009 -->
\{{#playground ./complex-example.rs}}
<!-- mdbook-lint-enable MDBOOK009 -->
````

## Troubleshooting

### Playground Not Working

1. Check `book.toml` configuration
2. Verify code compiles standalone
3. Test in actual Rust Playground
4. Check browser console for errors

### Common Error Messages

| Error | Solution |
|-------|----------|
| "Playground not configured" | Enable in book.toml |
| "Code doesn't compile" | Add missing imports/main function |
| "File not found" | Check file path in directive |
| "Invalid attribute" | Use supported attributes only |

## Related Rules

- [MDBOOK007](./mdbook007.html) - Include directive validation
- [MDBOOK008](./mdbook008.html) - Rustdoc include validation
- [MD040](../standard/md040.html) - Code block language tags

## References

- [mdBook - Rust Playground](https://rust-lang.github.io/mdBook/format/theme/editor.html)
- [Rust Playground](https://play.rust-lang.org/)
- [mdBook Configuration](https://rust-lang.github.io/mdBook/format/configuration/index.html)
