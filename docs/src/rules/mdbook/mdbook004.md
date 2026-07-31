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

```toml
[MDBOOK004]
case_sensitive = true                  # Case-sensitive comparison (default: true)
ignore_prefixes = ["Chapter", "Part"]  # Prefixes stripped before comparing (default: none)
```

- `case_sensitive`: with the default, "Setup" and "setup" are different titles. Set it
  to `false` to treat them as duplicates.
- `ignore_prefixes`: the first matching prefix is stripped from a title (along with the
  whitespace after it) before comparison, so "Chapter Setup" and "Setup" compare equal.

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
