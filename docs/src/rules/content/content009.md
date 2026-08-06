# CONTENT009 - No Excessive Heading Nesting

Heading nesting should not be too deep (default max: h4).

## Why This Rule Exists

Deep heading hierarchies (h5, h6) often signal that a chapter is trying to
hold too much at once. Readers lose track of where they are in the
document, sidebars and generated tables of contents become hard to scan,
and the content is usually better served by splitting it into separate
chapters or flattening the structure.

## Examples

### Incorrect

```markdown
# Chapter

## Section

### Subsection

#### Details

##### Too Deep
```

The `#####` heading is an h5, one level past the default maximum of h4.

### Correct

```markdown
# Chapter

## Section

### Subsection

#### Details
```

Headings stay at h4 or shallower, so nothing is flagged.

## Configuration

```toml
[CONTENT009]
# Deepest heading level allowed. Clamped to 1-6. Default: 4.
max_depth = 4
```

Both `max_depth` and `max-depth` are accepted in configuration. Any value
outside 1-6 is clamped to that range.

```toml
[CONTENT009]
max_depth = 2
```

With `max_depth = 2`, an `### Subsection` (h3) heading is reported.

## When to Disable

- Reference material (API docs, generated changelogs) that legitimately needs
  deep, granular subsections
- Chapters imported from external sources with an existing deep hierarchy
  you don't want to restructure right now

## Rule Details

- **Rule ID**: CONTENT009
- **Aliases**: no-excessive-nesting
- **Category**: Content
- **Severity**: Warning
- **Auto-fix**: No

## Related Rules

- [CONTENT005](./content005.md) - Heading immediately followed by a subheading
- [MD001](../standard/md001.md) - Heading levels should only increment by one level at a time
- [MD003](../standard/md003.md) - Heading style consistency
