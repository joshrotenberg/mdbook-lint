# MDBOOK004 - No Duplicate Chapter Titles

**Severity**: Warning  
**Category**: mdBook-specific  
**Auto-fix**: Not available

## Rule Description

This rule ensures that chapter titles are unique across the entire mdBook project. Duplicate titles can confuse readers and make navigation difficult.

## Why This Rule Exists

Unique chapter titles are important because:
- Prevents reader confusion when navigating the book
- Ensures clear distinction between different chapters
- Improves search functionality and indexing
- Makes cross-references unambiguous
- Helps maintain organized documentation structure

## Examples

### ❌ Incorrect (violates rule)

In `chapter1.md`:
```markdown
# Introduction
Content for first introduction...
```

In `chapter5.md`:
```markdown
# Introduction
Different content but same title...
```

### ✅ Correct

In `chapter1.md`:
```markdown
# Getting Started
Introduction content...
```

In `chapter5.md`:
```markdown
# API Introduction
API-specific introduction...
```

Or use more specific titles:
```markdown
# Project Overview
# Installation Guide
# Configuration Reference
# API Documentation
```

## What This Rule Checks

1. **First-level headings**: Checks all H1 headings across files
2. **Case sensitivity**: Treats "Introduction" and "introduction" as duplicates
3. **Cross-file validation**: Compares titles across all book chapters
4. **SUMMARY.md entries**: Validates chapter titles in the table of contents

## Configuration

```toml
[rules.MDBOOK004]
case_sensitive = false  # Case-sensitive comparison (default: false)
ignore_prefixes = ["Chapter", "Part"]  # Prefixes to ignore
```

## Common Issues and Solutions

### Issue: Generic Titles
Many chapters use generic titles like "Introduction" or "Overview":

```markdown
<!-- Bad: Too generic -->
# Introduction
# Overview
# Configuration
# Usage

<!-- Good: More specific -->
# Project Introduction
# Architecture Overview
# Database Configuration
# CLI Usage
```

### Issue: Section Titles as Chapter Titles
Using section-level titles as chapter titles:

```markdown
<!-- Bad: Section-like titles -->
# Installing
# Configuring
# Running

<!-- Good: Complete chapter titles -->
# Installation Guide
# Configuration Reference
# Running the Application
```

## When to Disable

Consider disabling this rule if:
- Your book intentionally uses duplicate titles (e.g., multiple "Introduction" sections)
- You're generating content dynamically with potential duplicates
- You have a large multi-part book where context makes duplicates clear

### Disable in Config

```toml
# .mdbook-lint.toml
disabled_rules = ["MDBOOK004"]
```

### Disable for Specific Files

```toml
[[overrides]]
path = "appendices/**"
disabled_rules = ["MDBOOK004"]
```

## Best Practices

1. **Be specific**: Use descriptive, unique titles for each chapter
2. **Add context**: Include the subject area in the title
3. **Use hierarchy**: Let SUMMARY.md structure provide context
4. **Consider prefixes**: Use part/section prefixes for clarity

### Title Patterns

```markdown
# [Subject] + [Type]
# Database Configuration
# API Reference
# Testing Guide

# [Action] + [Target]
# Installing Dependencies
# Configuring the Server
# Building from Source

# [Module] + [Aspect]
# Authentication Overview
# Storage Implementation
# Network Architecture
```

## Related Rules

- [MDBOOK003](./mdbook003.html) - SUMMARY.md structure validation
- [MD024](../standard/md024.html) - Multiple headings with same content
- [MD025](../standard/md025.html) - Multiple top-level headings

## References

- [mdBook - SUMMARY.md](https://rust-lang.github.io/mdBook/format/summary.html)
- [mdBook Structure](https://rust-lang.github.io/mdBook/guide/creating.html)