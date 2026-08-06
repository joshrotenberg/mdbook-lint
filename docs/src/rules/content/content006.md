# CONTENT006 - No Broken Internal Links

Internal anchor links (`[text](#anchor)`) should point to a heading that
actually exists in the same document.

## Why This Rule Exists

Markdown lets you link to a heading within the same file using
`[text](#anchor)`, where `anchor` is the heading's generated slug. If the
heading is renamed, removed, or the anchor is mistyped, the link silently
breaks: mdBook renders it as a dead link with no warning at build time.

CONTENT006 generates the same anchor slugs mdBook does and checks every
in-page anchor link against them, so broken links are caught before publish.

Only links of the form `(#anchor)` are checked. Links to other files
(`./other.md`), links to another file's anchor (`./other.md#section`), and
external URLs are out of scope, since this rule can only see headings in the
current document.

## Examples

### Incorrect

```markdown
# Getting Started

See [broken link](#nonexistent-section) for more info.
```

The document has no `## Nonexistent Section` heading, so `#nonexistent-section`
does not resolve to anything.

### Correct

```markdown
# Getting Started

See [the introduction](#getting-started) for more info.

## Installation

Check [installation](#installation) instructions.
```

Each anchor matches a heading's generated slug: `# Getting Started` becomes
`getting-started`, `## Installation` becomes `installation`.

### Duplicate headings

When the same heading text appears more than once, mdBook disambiguates the
slugs by appending `-1`, `-2`, and so on to the second and later occurrences.
CONTENT006 follows the same numbering:

```markdown
## Topic

Look at its [details](#details).

### Details

Details about the topic.

## Another Topic

Look at its [details](#details-1).

### Details

Details about the other topic.
```

### Links inside code blocks are ignored

````markdown
# Title

```markdown
[example](#nonexistent)
```
````

Both the fenced code block's content and any heading-like text inside it are
skipped, so example snippets don't trigger false positives and can't be used
as link targets.

## Configuration

This rule has no configuration options.

## When to Disable

- Documents that rely on anchors injected by a template or preprocessor
  after mdBook-lint runs, which this rule cannot see
- Books using a non-default slug scheme that doesn't match mdBook's

## Rule Details

- **Rule ID**: CONTENT006
- **Aliases**: no-broken-internal-links
- **Category**: Content
- **Severity**: Warning
- **Auto-fix**: No

## Related Rules

- [CONTENT005](./content005.md) - A heading followed straight by a subheading
- [MD042](../standard/md042.md) - Empty links
- [MD051](../standard/md051.md) - Link fragments
