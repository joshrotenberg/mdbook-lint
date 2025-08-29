# MDBOOK003 - Invalid SUMMARY.md Structure

**Severity**: Error  
**Category**: mdBook-specific  
**Auto-fix**: Not available

## Rule Description

This rule validates that `SUMMARY.md` follows mdBook's required structure and conventions. The SUMMARY.md file defines your book's table of contents and must follow specific formatting rules.

## Why This Rule Exists

Proper SUMMARY.md structure is essential because:
- mdBook uses it to generate navigation
- Incorrect structure causes build failures
- Defines the reading order of chapters
- Controls the book's hierarchical organization
- Enables proper sidebar navigation

## Examples

### ❌ Incorrect (violates rule)

```markdown
# Summary

[Introduction](./introduction.md)

- Part 1
  - [Chapter 1](./chapter1.md)
  
* [Chapter 2](./chapter2.md)  <!-- Mixed list markers -->

  - [Chapter 3](./chapter3.md)  <!-- Incorrect indentation -->
  
[](./empty.md)  <!-- Empty link text -->

- - [Double nested](./nested.md)  <!-- Invalid nesting -->
```

### ✅ Correct

```markdown
# Summary

[Introduction](./introduction.md)

# User Guide

- [Getting Started](./getting-started.md)
  - [Installation](./installation.md)
  - [Configuration](./configuration.md)
- [Advanced Usage](./advanced.md)

# Reference

- [API Documentation](./api.md)
- [Configuration Reference](./config-ref.md)

---

[Contributors](./contributors.md)
```

## SUMMARY.md Structure Rules

### Required Elements

1. **Title**: Must start with `# Summary`
2. **Prefix Chapter**: Optional `[Introduction](./intro.md)` before numbered chapters
3. **Numbered Chapters**: Use consistent list markers (`-` or `*`)
4. **Suffix Chapters**: Optional unnumbered chapters after separator

### Formatting Rules

- **Consistent indentation**: Use 2 or 4 spaces per level
- **Consistent list markers**: Use either `-` or `*` throughout
- **Valid links**: All links must have text and valid paths
- **Proper nesting**: Child chapters indented under parents
- **Part headers**: Use `# Part Name` for sections

### Special Elements

```markdown
# Summary

[Preface](./preface.md)          <!-- Prefix chapter -->

# Part I

- [Chapter 1](./ch1.md)           <!-- Numbered chapter -->
  - [Section 1.1](./ch1-1.md)     <!-- Nested chapter -->
- [Chapter 2](./ch2.md)
  - [Draft]()                     <!-- Draft chapter (no link) -->

---                               <!-- Separator -->

[Appendix A](./appendix-a.md)    <!-- Suffix chapter -->
```

## Configuration

```toml
[MDBOOK003]
allow_draft_chapters = true    # Allow chapters without links (default: true)
require_part_headers = false   # Require part headers (default: false)
max_depth = 3                   # Maximum nesting depth (default: 3)
```

## Common Issues and Solutions

### Issue: Mixed List Markers
```markdown
<!-- Wrong -->
- [Chapter 1](./ch1.md)
* [Chapter 2](./ch2.md)

<!-- Correct -->
- [Chapter 1](./ch1.md)
- [Chapter 2](./ch2.md)
```

### Issue: Incorrect Indentation
```markdown
<!-- Wrong -->
- [Chapter 1](./ch1.md)
   - [Section](./sec.md)  <!-- 3 spaces -->

<!-- Correct -->
- [Chapter 1](./ch1.md)
  - [Section](./sec.md)   <!-- 2 spaces -->
```

### Issue: Invalid Draft Syntax
```markdown
<!-- Wrong -->
- [TODO](.)
- [Draft](/)

<!-- Correct -->
- [Draft]()
```

## When to Disable

Consider disabling this rule if:
- You're using a custom mdBook theme with different requirements
- Your build process generates SUMMARY.md dynamically
- You're migrating from another documentation system

### Disable in Config

```toml
# .mdbook-lint.toml
disabled_rules = ["MDBOOK003"]
```

## Tips for Compliance

1. **Use consistent indentation**: Pick 2 or 4 spaces and stick with it
2. **Order matters**: Chapters appear in the order listed
3. **Test the build**: Run `mdbook build` to verify structure
4. **Use draft chapters**: For work-in-progress sections

## Related Rules

- [MDBOOK002](./mdbook002.html) - Invalid internal link
- [MDBOOK005](./mdbook005.html) - Orphaned files
- [MDBOOK025](./mdbook025.html) - SUMMARY.md heading structure

## References

- [mdBook - SUMMARY.md Format](https://rust-lang.github.io/mdBook/format/summary.html)
- [mdBook Configuration](https://rust-lang.github.io/mdBook/format/configuration/index.html)