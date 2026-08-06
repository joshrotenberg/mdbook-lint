# CONTENT011 - No Future Tense

Documentation should use present tense instead of future tense.

## Why This Rule Exists

Technical documentation describes what software does, not what it will do at
some later point. "This function will return an integer" reads as a promise
about a future release, while "This function returns an integer" states a
fact about the current behavior. Present tense is shorter, more direct, and
avoids ambiguity about whether a feature is implemented yet.

## Examples

### Incorrect

```markdown
This function will return an integer.

The value will be updated.

This will throw an exception if the input is invalid.

This method is going to create a new file.

These functions are going to process the data.
```

### Correct

```markdown
This function returns an integer.

The value is updated.

This throws an exception if the input is invalid.

This method creates a new file.

These functions process the data.
```

The rule only matches "will" directly followed by one of a known set of
verbs (see below), so unrelated uses of "will" are left alone:

```markdown
The user's free will is respected.
```

## Configuration

This rule has no configuration options.

## When to Disable

- Documentation that intentionally describes planned or upcoming behavior,
  such as a roadmap or changelog "Unreleased" section
- API references written for a not-yet-released version, where future tense
  correctly describes functionality that does not exist yet

## Rule Details

- **Rule ID**: CONTENT011
- **Aliases**: no-future-tense
- **Category**: Content
- **Severity**: Info
- **Auto-fix**: No

## Flagged Patterns

The rule matches, case-insensitively:

- `will` followed directly by one of a fixed list of verbs: `be`, `have`,
  `return`, `throw`, `create`, `generate`, `produce`, `output`, `display`,
  `show`, `print`, `log`, `emit`, `trigger`, `fire`, `call`, `invoke`,
  `execute`, `run`, `start`, `stop`, `open`, `close`, `read`, `write`,
  `load`, `save`, `send`, `receive`, `get`, `set`, `add`, `remove`,
  `delete`, `update`, `change`, `modify`, `process`, `handle`, `validate`,
  `check`, `verify`, `parse`, `convert`, `transform`, `format`, `render`,
  `build`, `compile`, `install`, `download`, `upload`, `fetch`, `request`,
  `respond`
- `is going to <word>`
- `are going to <word>`

Each violation message includes a suggested present-tense rewrite, for
example `will return` becomes `return` and `will be` becomes `is`.

At most one violation is reported per line, even if the line contains
several future-tense phrases.

Fenced code blocks (`` ``` `` or `~~~`) are tracked across lines and skipped
in full, so a Rust doc comment like `// This function will return a value`
inside a code sample is not flagged.

HTML comments and mdbook directives are not tracked. The rule skips a line
only when the trimmed line *begins with* `<!--` or `{{#`; it keeps no
comment state and does not mask matches within a line. Future tense is
still reported inside a multi-line HTML comment, after an inline
`<!-- ... -->`, or on a line where a `{{#...}}` directive follows other
text:

```markdown
<!-- This whole line will be ignored, because the line starts with "<!--". -->

<!--
This function will return an integer.   <- still flagged
-->

Some text <!-- this will throw an error --> more text.   <- still flagged

See the sample {{#playground example.rs}} and it will create a file.   <- still flagged

{{#playground example.rs}} and this will delete things.   <- skipped, line starts with "{{#"
```

## Related Rules

- [CONTENT001](./content001.md) - TODO, FIXME and similar markers left in prose
- [CONTENT002](./content002.md) - Placeholder text such as lorem ipsum
