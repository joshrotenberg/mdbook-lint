# MDBOOK023 - Chapter Title Matching

The link title used for a chapter in `SUMMARY.md` should match the H1
header in the linked file.

## Why This Rule Exists

mdBook's navigation sidebar shows the title from `SUMMARY.md`, while the
page itself opens with its own H1. When the two disagree, readers see one
title in the sidebar and land on a page that calls itself something else,
which reads as a broken or stale link even though the link itself works
fine.

## Examples

### Incorrect

`src/SUMMARY.md`:

```markdown
# Summary

- [Getting Started](intro.md)
```

`src/intro.md`:

```markdown
# Introduction to the Project

Welcome!
```

The sidebar says "Getting Started" but the page opens with "Introduction to
the Project".

### Correct

`src/SUMMARY.md`:

```markdown
# Summary

- [Getting Started](intro.md)
```

`src/intro.md`:

```markdown
# Getting Started

Welcome!
```

Matching is case-insensitive and normalizes whitespace, so `# getting
started` and `# Getting  Started` are both accepted against a SUMMARY.md
entry of `Getting Started`.

## Configuration

This rule has no configuration options.

## When to Disable

- Chapters that intentionally use a shorter or reframed title in the
  navigation than in the page's own heading

## Rule Details

- **Rule ID**: MDBOOK023
- **Aliases**: chapter-title-match
- **Category**: MdBook
- **Severity**: Warning
- **Auto-fix**: No

## Scope

This rule only checks `SUMMARY.md`. For each chapter link it finds, it
resolves the linked file relative to the book's source directory and
compares the link text to that file's first H1 header:

- Draft chapters (`[Title]()`, empty path) are skipped.
- External links (`http://`, `https://`) and anchor-only links (`#section`)
  are skipped.
- If the linked file doesn't exist, that's reported by
  [MDBOOK002](./mdbook002.md), not here.
- If the linked file has no H1 header at all, that's reported by
  [MD041](../standard/md041.md), not here — this rule only compares titles
  when both sides are present.

## Related Rules

- [MDBOOK002](./mdbook002.md) - Invalid internal link
- [MDBOOK021](./mdbook021.md) - Single title directive per chapter
- [MDBOOK022](./mdbook022.md) - Title directive near top
- [MD041](../standard/md041.md) - First line in a file should be a top-level heading
