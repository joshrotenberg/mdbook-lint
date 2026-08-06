# CONTENT010 - Link Text Quality

Link text should be descriptive, not generic like "click here" or "here".

## Why This Rule Exists

Screen readers often present a page's links as a standalone list, out of the
surrounding sentence. Link text like "here" or "click here" gives no
information in that context, and it forces every reader to hunt through the
surrounding paragraph to find out where a link actually leads.

## Examples

### Incorrect

```markdown
[Click here](https://example.com) to learn more.

For more information, see [here](./docs.md).

Follow [this link](https://example.com) for details.

[Read more](./article.md)

[Learn more](https://docs.example.com)

See [this](./example.md) for an example.

[Info](./help.md)

For [details](./spec.md), see the specification.
```

### Correct

```markdown
Check out the [installation guide](./install.md) for details.

See the [API documentation](https://docs.example.com) for more info.

Read [more about configuration](./config.md) here.
```

The last example is not flagged: the rule matches link text only when it is
*exactly* one of the generic phrases, so "more about configuration" (which
merely contains "more") passes.

## Configuration

This rule has no configuration options.

## When to Disable

- Content where the surrounding UI text is itself "click here" or similar,
  and the link text is meant to mirror it
- Legacy content being migrated incrementally, where rewriting every link
  is out of scope for the current change

## Rule Details

- **Rule ID**: CONTENT010
- **Aliases**: link-text-quality
- **Category**: Content
- **Severity**: Warning
- **Auto-fix**: No

## Flagged Phrases

The rule performs a case-insensitive exact match of a link's text against
this list:

- "click here"
- "here"
- "this link"
- "this page"
- "this article"
- "this"
- "link"
- "read more"
- "more"
- "learn more"
- "see more"
- "more info"
- "more information"
- "details"
- "info"

Links inside fenced code blocks are ignored.

## Related Rules

- [MD042](../standard/md042.md) - Empty links
- [MD059](../standard/md059.md) - Descriptive link text
