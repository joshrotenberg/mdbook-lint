# MDBOOK025 - Multiple H1 Headings Allowed in SUMMARY.md

**Severity**: Info  
**Category**: mdBook-specific  
**Auto-fix**: Not available

## Rule Description

This rule specifically allows multiple H1 headings in SUMMARY.md files while still enforcing the single H1 rule (MD025) in regular chapter files. SUMMARY.md uses H1 headings to define part separators in the book structure.

## Why This Rule Exists

SUMMARY.md has special requirements because:

- H1 headings define book parts/sections
- Multiple parts are common in large books
- mdBook treats these H1s as structural elements
- They don't represent document headings but navigation structure
- Standard MD025 rule would incorrectly flag valid SUMMARY.md files

## Examples

### ✅ Correct SUMMARY.md Structure

```markdown
# Summary

[Introduction](./introduction.md)

# Part I: Getting Started

- [Installation](./chapter1/installation.md)
- [Configuration](./chapter1/configuration.md)

# Part II: User Guide

- [Basic Usage](./chapter2/basic-usage.md)
- [Advanced Features](./chapter2/advanced.md)

# Part III: Reference

- [API Documentation](./chapter3/api.md)
- [Configuration Reference](./chapter3/config-ref.md)

---

[Appendix A](./appendix-a.md)
[Appendix B](./appendix-b.md)
```

### ❌ What This Rule Prevents

In regular chapter files (not SUMMARY.md):

```markdown
# First Heading

Content...

# Second H1 Heading  <!-- MD025 violation in regular files -->

More content...
```

## SUMMARY.md Structure Rules

### Part Headers

- **H1 headings** (`# Part Name`) create part divisions
- Parts group related chapters
- Part headers appear in the rendered navigation
- No limit on number of parts

### Special Elements

```markdown
# Summary                         <!-- Required first line -->

[Prefix Chapter](./preface.md)   <!-- Before numbered chapters -->

# Part Name                       <!-- Part header -->

- [Chapter](./ch.md)              <!-- Numbered chapters -->
  - [Section](./sect.md)          <!-- Nested chapters -->

---                               <!-- Separator -->

[Suffix Chapter](./appendix.md)  <!-- After numbered chapters -->
```

## Configuration

```toml
[MDBOOK025]
# This rule has no configuration options
# It automatically applies only to SUMMARY.md
```

## How It Works

This rule:

1. Detects if the file is SUMMARY.md
2. Allows multiple H1 headings in SUMMARY.md
3. Defers to MD025 for all other files
4. Validates proper SUMMARY.md structure

## Common Patterns

### Book with Multiple Parts

```markdown
# Summary

[Preface](./preface.md)

# Part I: Fundamentals

- [Chapter 1](./ch1.md)
- [Chapter 2](./ch2.md)

# Part II: Intermediate

- [Chapter 3](./ch3.md)
- [Chapter 4](./ch4.md)

# Part III: Advanced

- [Chapter 5](./ch5.md)
- [Chapter 6](./ch6.md)
```

### Book without Parts

```markdown
# Summary

[Introduction](./intro.md)

- [Chapter 1](./ch1.md)
- [Chapter 2](./ch2.md)
- [Chapter 3](./ch3.md)

---

[Conclusion](./conclusion.md)
```

### Mixed Structure

```markdown
# Summary

- [Getting Started](./start.md)

# Core Concepts

- [Fundamentals](./fundamentals.md)
- [Architecture](./architecture.md)

# Advanced Topics

- [Performance](./performance.md)
- [Security](./security.md)

---

[Glossary](./glossary.md)
```

## Best Practices

1. **Use parts for organization**: Group related chapters
2. **Keep part names concise**: They appear in navigation
3. **Order matters**: Parts appear in sequence
4. **Be consistent**: Use similar naming patterns
5. **Consider reader flow**: Logical progression through parts

### Part Naming Conventions

```markdown
<!-- Numbered parts -->
# Part I: Introduction
# Part II: Core Concepts
# Part III: Advanced Topics

<!-- Descriptive parts -->
# Getting Started
# User Guide
# API Reference
# Appendices

<!-- Module-based -->
# Core Modules
# Extension Modules
# Utility Modules
```

## Interaction with Other Rules

### Works With

- **MDBOOK003**: Validates overall SUMMARY.md structure
- **MD022**: Headings surrounded by blank lines
- **MD026**: No trailing punctuation in headings

### Overrides

- **MD025**: Multiple top-level headings (in SUMMARY.md only)

## When to Disable

Consider disabling this rule if:

- You use a custom book structure
- You have a different table of contents format
- You don't use SUMMARY.md
- You prefer strict MD025 enforcement everywhere

### Disable in Config

```toml
# .mdbook-lint.toml
disabled_rules = ["MDBOOK025"]

# Or disable both MDBOOK025 and MD025
disabled_rules = ["MDBOOK025", "MD025"]
```

## Tips

1. **Part headers are optional**: Not every book needs parts
2. **Unnumbered chapters**: Can exist before first part
3. **Separator sections**: Use `---` for appendices
4. **Draft chapters**: Use `[Chapter]()` for placeholders
5. **Nested structure**: Indent with 2 or 4 spaces consistently

## Related Rules

- [MDBOOK003](./mdbook003.html) - SUMMARY.md structure validation
- [MD025](../standard/md025.html) - Multiple top-level headings (general rule)
- [MD001](../standard/md001.html) - Heading levels should increment

## References

- [mdBook - SUMMARY.md](https://rust-lang.github.io/mdBook/format/summary.html)
- [mdBook - Parts](https://rust-lang.github.io/mdBook/format/summary.html#parts)
