# CONTENT004 - Heading Capitalization Consistency

Headings should use a consistent capitalization style throughout a document.

## Why This Rule Exists

Mixing Title Case and sentence case headings in the same document reads as
unpolished, and it's usually accidental: a section gets added later, written
by someone else, or pasted from a different source. Consistent heading
capitalization makes a document feel like it was written by one voice.

## Examples

### Incorrect

The first heading establishes Title Case as the document's style, so the
lowercase second heading is flagged:

```markdown
# Getting Started Guide

## installation steps

### More Configuration Options
```

### Correct

All headings follow the same style. Title Case:

```markdown
# Getting Started Guide

## Installation Steps

### Configuration Options
```

Sentence case works equally well, as long as it's consistent:

```markdown
# Getting started guide

## Installation steps

### Configuration options
```

## How Style Is Detected

By default (`style = "consistent"`), the rule doesn't require Title Case or
sentence case specifically. It detects whichever style the first
multi-word heading uses and expects every later heading to match it.

A few details affect how a heading is classified:

- **Single-word headings are skipped.** `# Introduction` says nothing about
  capitalization style either way.
- **Acronyms are ignored when judging a word's case.** `API`, `HTTP`, and
  similar all-uppercase words don't count against sentence case.
- **Articles, conjunctions, and short prepositions** (`a`, `the`, `and`,
  `of`, `to`, `vs`, etc.) are excluded from the Title Case check, since Title
  Case conventionally leaves them lowercase.
- **A heading that's valid under both styles doesn't set or break the
  baseline.** For example, `# Agentic SDLC` is valid Title Case and valid
  sentence case at once, because its only non-first word is an acronym. Such
  a heading is never itself flagged, and if it's the first heading in the
  document, the rule keeps looking for the next heading to establish the
  baseline instead of locking onto it.

Headings inside fenced code blocks are not checked.

## Configuration

```toml
[CONTENT004]
# "consistent" (default): match whatever style the first heading uses.
# "title" / "title_case": require Title Case throughout.
# "sentence" / "sentence_case": require sentence case throughout.
style = "consistent"
```

### `style`

With `"title"`, every multi-word heading must be Title Case:

```markdown
# Getting started guide
```

```
Heading 'Getting started guide' should use Title Case
```

With `"sentence"`, every multi-word heading must be sentence case:

```markdown
# Getting Started Guide
```

```
Heading 'Getting Started Guide' should use sentence case
```

## When to Disable

- Documents that intentionally mix heading styles, such as ones that quote
  headings from external sources verbatim
- Books where headings are short enough that capitalization style rarely
  comes up

## Rule Details

- **Rule ID**: CONTENT004
- **Category**: Content
- **Severity**: Warning
- **Auto-fix**: No

## Related Rules

- [CONTENT005](./content005.md) - A heading followed straight by a subheading
- [CONTENT009](./content009.md) - Headings nested deeper than a reader will follow
- [MD001](../standard/md001.md) - Heading levels should only increment by one level at a time
- [MD003](../standard/md003.md) - Heading style
