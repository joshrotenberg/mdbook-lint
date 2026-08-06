# CONTENT003 - Short Chapters

Chapters should have enough content to be worth their own page.

## Why This Rule Exists

A chapter with only a sentence or two is often a stub: a heading was added to
`SUMMARY.md` before the content was written, or a section was split out and
never filled in. This rule counts the words in a chapter and flags any that
fall below a configurable minimum (50 by default), which surfaces
work-in-progress content before it ships in the book.

## Examples

### Incorrect

```markdown
# Short Chapter

This is too short.
```

<!-- 4 words of body content, well under the 50-word minimum -->

### Correct

```markdown
# Chapter Title

This is a paragraph with enough content to pass the minimum word count
threshold. We need to write several sentences here to make sure we have at
least fifty words in total. Let me add some more text to ensure we
definitely pass the check. Here is another sentence. And another one. Plus a
few more words to be safe.
```

### What Counts Toward the Word Total

The count is line based. Outside code blocks, a line is dropped from the total
when it is a fence line, or when its first non-whitespace characters are `#`,
`<!--`, or `{{#`. Every other line contributes all of its whitespace-separated
tokens. An ATX heading, an HTML comment that opens and closes on its own line,
and an mdBook directive on its own line are therefore not counted:

```markdown
# This Heading Has Many Words In It

<!-- This HTML comment is also not counted -->

\{{#include file.rs}}

Short body.
```

The chapter above is still flagged: only "Short body." counts, for a total of
two words.

Because that is a prefix test on each line rather than an understanding of the
markup, three ordinary constructs do count as prose:

- **Setext headings.** Only `#` headings are recognized. A heading underlined
  with `===` or `---` contributes its own words plus one more for the underline
  line, so a long setext heading can carry a stub past the threshold. The
  chapter above, with its heading rewritten as setext, counts ten words rather
  than two.
- **Multi-line HTML comments.** Only the opening line is skipped. Every
  continuation line, up to and including the one holding `-->`, is counted, so
  a stub padded with a long comment can pass the rule.
- **Comments and directives that do not start their line.** The line below
  counts three words, because `\{{#include` and `file.rs}}` are counted
  alongside `Body.`:

  ```markdown
  Body. \{{#include file.rs}}
  ```

Code blocks are excluded by default, so a chapter that is mostly a code
sample still needs prose around it to pass:

````markdown
# Chapter

Short intro.

```rust
fn main() {
    // Code comments don't count toward the word total by default
    println!("Hello");
}
```
````

## Configuration

```toml
[CONTENT003]
# Minimum word count below which a chapter is flagged. Default: 50.
min_words = 50

# Whether to count words inside code blocks. Default: false.
include_code_blocks = false
```

### `min_words`

Lower it for books with intentionally terse chapters, or raise it to demand
more substantial pages:

```toml
[CONTENT003]
min_words = 20
```

### `include_code_blocks`

With the default `false`, words inside fenced code blocks (` ``` ` or `~~~`)
are not counted. Set it to `true` to count code toward the total, which is
useful for reference chapters that are mostly runnable examples:

```toml
[CONTENT003]
include_code_blocks = true
```

Even with `true`, two kinds of line stay out of the total: the fence lines
themselves, and code lines beginning with `#`, since the heading test is
applied to code lines too. A shell or Python comment on a line of its own is
therefore skipped, but a trailing comment after code is not: the test is on
the start of the line, so `print("x")  # note` contributes every token on it.

## When to Disable

- Reference chapters that are intentionally brief, such as a single command's
  options
- Landing pages that link out to subsections rather than containing prose
  themselves

## Rule Details

- **Rule ID**: CONTENT003
- **Aliases**: no-short-chapters
- **Category**: Content
- **Severity**: Warning
- **Auto-fix**: No

## Related Rules

- [CONTENT001](./content001.md) - TODO and FIXME markers left in prose
- [CONTENT002](./content002.md) - Placeholder text such as lorem ipsum
