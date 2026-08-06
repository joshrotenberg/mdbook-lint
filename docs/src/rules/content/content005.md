# CONTENT005 - Introductory Paragraph Before Subheading

Chapters should have introductory content before the first subheading.

## Why This Rule Exists

A chapter that jumps straight from its title into a subheading gives the
reader no framing for what the chapter covers. A short introduction after the
H1 orients the reader before the first H2 (or deeper) heading takes over.

## Examples

### Incorrect

```markdown
# Chapter Title

## First Section
```

```markdown
# Chapter Title

Brief intro.

## First Section
```

### Correct

```markdown
# Chapter Title

This chapter covers important topics that you need to understand.
We will explore several key concepts in detail below.

## First Section
```

Only the introduction before the *first* subheading is checked. Chapters with
no subheadings at all, and chapters with no H1, are not checked:

```markdown
# Chapter Title

This is a simple chapter with no subheadings.
It just has regular paragraphs of content.
```

```markdown
## First Section

Some content here.

## Second Section
```

Code blocks, HTML comments, and mdBook directives such as `{{#include}}` do
not count toward the word total, since they are not prose written for this
chapter:

````markdown
# Chapter Title

```rust
// This code block should not count as intro
fn main() {}
```

## First Section
````

## Configuration

```toml
[CONTENT005]
# Minimum number of words required in the introduction. Default: 10.
min_words = 10
```

### `min_words`

`min_intro_words` is accepted as an alias, and both `snake_case` and
`kebab-case` keys work (`min-words`, `min-intro-words`). Words are counted
from the line after the H1 up to (but not including) the first subheading,
skipping code blocks, HTML comments, and mdBook directives.

## When to Disable

- Chapters that are intentionally just a list or a table of links, with the
  context provided elsewhere in the book
- Generated reference pages where a subheading immediately following the
  title is the expected shape

## Rule Details

- **Rule ID**: CONTENT005
- **Aliases**: intro-before-subheading
- **Category**: Content
- **Severity**: Warning
- **Auto-fix**: No

## Related Rules

- [CONTENT003](./content003.md) - Chapters too short to be worth a page
- [CONTENT009](./content009.md) - Headings nested deeper than a reader will follow
- [MD022](../standard/md022.md) - Headings should be surrounded by blank lines
