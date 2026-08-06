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
# Deepest heading level allowed. Default: 4.
max_depth = 4
```

Both `max_depth` and `max-depth` are accepted in configuration.

Only a non-negative TOML integer is read. Such a value is clamped to 1-6, so
`0` behaves as `1` and `99` behaves as `6`. Anything else is discarded
without an error and the rule runs at the default of 4: that includes
negative integers, so `max_depth = -1` does not mean "the strictest setting",
it means the same as leaving the option out. Quoted numbers (`"2"`) and
floats (`2.0`) are dropped the same way.

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
