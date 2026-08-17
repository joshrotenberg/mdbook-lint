# MDBOOK021 - Single Title Directive Per Chapter

`\{{#title}}` directive should appear only once per chapter.

## Why This Rule Exists

The `\{{#title}}` directive sets the page title shown in the browser tab.
When a chapter contains more than one, mdBook doesn't merge or reject the
extras: only one directive takes effect (usually the last one), and the
others sit in the file doing nothing, misleading anyone editing the chapter
about which title is actually in effect.

## Examples

### Incorrect

```text
\{{#title First Title}}

# Chapter

\{{#title Second Title}}

Content.
```

The directive on line 5 is flagged as a duplicate, with the message pointing
back to the first occurrence on line 1.

```text
\{{#title First}}
\{{#title Second}}
\{{#title Third}}
```

Every occurrence after the first is flagged, so this reports two violations.

### Correct

```text
\{{#title My Page Title}}

# Chapter Title

Content.
```

A chapter with no `\{{#title}}` directive at all is also correct; this rule
only fires when more than one directive is present.

## Scope

This rule does not skip fenced code blocks. It scans each raw line for the
directive pattern, so a `\{{#title}}` directive shown as an example inside a
code block is still counted; the rule takes that position on the grounds that
mdBook's preprocessor would otherwise process it there too.

Escaping does not exempt an example. The pattern the rule matches,
`\{\{#title\s+[^}]+\}\}`, makes no allowance for a preceding backslash, so an
escaped directive still matches, starting at the `{{` and reported one column
to the right of an unescaped one. Escaping keeps mdBook from rendering the
directive, but this rule counts it either way. The escaped examples on this
page are counted, and linting this page reports MDBOOK021 for each of them
after the first.

Matching requires text after `#title` before the closing `}}`, so a bare
mention like `` `\{{#title}}` `` in prose is not treated as a directive and
is not counted.

## Configuration

This rule has no configuration options.

## Rule Details

- **Rule ID**: MDBOOK021
- **Aliases**: single-title-directive
- **Category**: MdBook
- **Severity**: Warning
- **Stability**: Stable
- **Auto-fix**: No

## Related Rules

- [MDBOOK022](./mdbook022.md) - Title directive near top
- [MDBOOK023](./mdbook023.md) - Chapter title matching
