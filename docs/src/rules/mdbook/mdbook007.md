# MDBOOK007 - Invalid Include Directive

**Severity**: Error  
**Category**: mdBook-specific  
**Auto-fix**: Not available

## Rule Description

This rule validates `{{#include}}` directives used to include content from other files. It ensures the syntax is correct and the referenced files exist.

## Why This Rule Exists

Valid include directives are essential because:
- Prevents build failures from broken includes
- Ensures included content is available
- Maintains documentation modularity
- Enables content reuse across chapters
- Helps identify moved or renamed files

## Examples

### ❌ Incorrect (violates rule)

```markdown
<!-- File doesn't exist -->
{{#include ./missing-file.md}}

<!-- Invalid syntax -->
{{include ./file.md}}
{{#includes ./file.md}}

<!-- Malformed path -->
{{#include file.md}}  <!-- Missing ./ prefix -->
{{#include ../../../outside-project.md}}

<!-- Invalid line range syntax -->
{{#include ./file.md:1-}}
{{#include ./file.md:a-b}}
```

### ✅ Correct

```markdown
<!-- Include entire file -->
{{#include ./examples/sample.md}}

<!-- Include specific lines -->
{{#include ./code.rs:1:10}}
{{#include ./code.rs:5:}}
{{#include ./code.rs::10}}

<!-- Include with anchor -->
{{#include ./file.md:anchor}}

<!-- Include from parent directory -->
{{#include ../shared/header.md}}
```

## Include Directive Syntax

### Basic Include
```markdown
{{#include filepath}}
```

### Line Range Selection
```markdown
{{#include filepath:start:end}}  <!-- Lines start to end -->
{{#include filepath:start:}}     <!-- From start to EOF -->
{{#include filepath::end}}       <!-- From beginning to end -->
{{#include filepath:line}}       <!-- Single line -->
```

### Anchor-based Include
```markdown
{{#include filepath:anchor}}     <!-- Include anchor section -->
```

In the source file:
```markdown
<!-- ANCHOR: example -->
Content to include
<!-- ANCHOR_END: example -->
```

## Configuration

```toml
[MDBOOK007]
check_line_ranges = true    # Validate line numbers exist (default: true)
allow_external = false      # Allow includes outside src/ (default: false)
max_depth = 3              # Maximum directory traversal depth (default: 3)
```

## Common Issues and Solutions

### Issue: Relative Path Confusion
```markdown
<!-- Current file: src/chapter1/section.md -->

<!-- Wrong: Assumes path from src/ -->
{{#include examples/code.rs}}  ✗

<!-- Correct: Relative to current file -->
{{#include ../examples/code.rs}}  ✓
```

### Issue: Line Numbers Out of Range
```markdown
<!-- file.md has 50 lines -->

<!-- Error: End line exceeds file length -->
{{#include ./file.md:1:100}}

<!-- Fix: Use open-ended range -->
{{#include ./file.md:1:}}
```

### Issue: Platform-Specific Paths
```markdown
<!-- Windows-style path (avoid) -->
{{#include .\examples\code.rs}}

<!-- Unix-style path (preferred) -->
{{#include ./examples/code.rs}}
```

## Advanced Usage

### Including Code with Hidden Lines

```rust
{{#include ./examples/main.rs}}
```

In `main.rs`:
```rust
# fn hidden_function() {
#     // This line is hidden in mdBook output
# }

fn visible_function() {
    // This is shown
}
```

### Nested Includes

Includes can contain other includes:
```markdown
<!-- chapter.md -->
{{#include ./sections/intro.md}}

<!-- sections/intro.md -->
This is the introduction.
{{#include ../examples/sample.md}}
```

## When to Disable

Consider disabling this rule if:
- You're generating included files during the build process
- You're migrating content with many broken includes
- You use a custom preprocessor that handles includes differently
- You're prototyping with placeholder includes

### Disable in Config

```toml
# .mdbook-lint.toml
disabled_rules = ["MDBOOK007"]
```

### Disable Inline

```markdown
<!-- mdbook-lint-disable MDBOOK007 -->
{{#include ./todo-file.md}}
<!-- mdbook-lint-enable MDBOOK007 -->
```

## Best Practices

1. **Use relative paths**: More maintainable than absolute paths
2. **Keep includes close**: Avoid deep directory traversal
3. **Document anchors**: Comment what each anchor contains
4. **Test after moving files**: Update include paths when reorganizing
5. **Prefer anchors over line numbers**: More stable than line ranges

### Include Organization

```
book/
├── src/
│   ├── chapter1.md
│   ├── chapter2.md
│   └── includes/        <!-- Dedicated includes directory -->
│       ├── header.md
│       ├── footer.md
│       └── examples/
│           └── code.rs
```

### Anchor Documentation

```markdown
<!-- ANCHOR: configuration_example -->
<!-- This anchor includes a complete configuration example -->
```toml
[book]
title = "My Book"
```
<!-- ANCHOR_END: configuration_example -->
```

## Error Messages

Common error messages and their solutions:

| Error | Solution |
|-------|----------|
| "File not found" | Check file path and spelling |
| "Invalid line range" | Verify line numbers exist |
| "Anchor not found" | Ensure anchor tags match |
| "Path traversal too deep" | Simplify directory structure |

## Related Rules

- [MDBOOK008](./mdbook008.html) - Rustdoc include validation
- [MDBOOK011](./mdbook011.html) - Template include syntax
- [MDBOOK012](./mdbook012.html) - Include line ranges

## References

- [mdBook - Including Files](https://rust-lang.github.io/mdBook/format/markdown.html#including-files)
- [mdBook Preprocessors](https://rust-lang.github.io/mdBook/format/configuration/preprocessors.html)