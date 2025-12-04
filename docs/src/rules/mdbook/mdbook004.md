# MDBOOK004 - No Duplicate Chapter Titles

Chapter titles should be unique across the book.

## Why This Rule Exists

Duplicate chapter titles create confusion in navigation and can cause issues
with mdBook's URL generation. Each chapter should have a distinct, identifiable
title.

## Examples

### Incorrect (SUMMARY.md)

```markdown
# Summary

- [Introduction](./intro.md)
- [Getting Started](./start.md)
- [Introduction](./advanced-intro.md)  <!-- Duplicate -->
```

### Correct

```markdown
# Summary

- [Introduction](./intro.md)
- [Getting Started](./start.md)
- [Advanced Introduction](./advanced-intro.md)
```

## Configuration

This rule has no configuration options.

## When to Disable

- Books with intentionally repeated section names
- Multi-part books where repetition is meaningful

## Rule Details

- **Rule ID**: MDBOOK004
- **Aliases**: no-duplicate-chapter-titles
- **Category**: MdBook
- **Severity**: Warning
- **Auto-fix**: No

## Impact

Duplicate titles can cause:

- Confusing navigation sidebar
- Ambiguous URL paths
- Search result confusion
- Poor user experience

## Related Rules

- [MD024](../standard/md024.md) - No duplicate headings
- [MDBOOK003](./mdbook003.md) - SUMMARY.md structure
- [MDBOOK025](./mdbook025.md) - SUMMARY.md heading structure
