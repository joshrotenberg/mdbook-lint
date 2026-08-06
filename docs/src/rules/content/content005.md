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

Fenced code blocks do not count toward the word total, since they are not
prose written for this chapter:

````markdown
# Chapter Title

```rust
// This code block should not count as intro
fn main() {}
```

## First Section
````

Apart from fenced blocks, the exclusion is a per-line test with no memory of
what came before: a line is dropped from the count only when its first
non-whitespace characters are `#`, `<!--`, or `{{#`. The last covers mdBook
directives such as `{{#include}}` on a line of their own, and the first covers
any line starting with a hash, which includes a second H1 as well as headings
below it. A single-line HTML comment is therefore skipped, but in a multi-line
comment only the opening line is: every body line and the closing `-->` count
as introduction prose, unless one of them happens to start with a prefix from
that same list. Indented (four-space) code blocks are counted too,
because each line is trimmed before it is tested and the indentation that
made it code is gone by then. Both of these chapters pass the rule despite
having no introduction:

```markdown
# Chapter Title

<!--
This comment body is not prose for the reader but it is counted anyway
-->

## First Section
```

```markdown
# Chapter Title

    let x = 1;
    let y = 2;
    let z = 3;
    println!("{} {} {}", x, y, z);

## First Section
```

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
skipping fenced code blocks in full and dropping any remaining line whose
first non-whitespace character is `#` or that begins with `<!--` or `{{#`.
As described above, that per-line test does not exclude the body of a
multi-line HTML comment or an indented code block.

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
