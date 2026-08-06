# CONTENT001 - No TODO Comments

TODO, FIXME, and similar work-in-progress markers should be resolved before publishing.

## Why This Rule Exists

Markers like `TODO`, `FIXME`, `XXX`, `HACK`, and `WIP` are useful while drafting, but
left in published documentation they read as unfinished work and erode trust in the
content. This rule flags them so they get resolved (or deliberately dismissed) before
a book ships.

## Examples

### Incorrect

```markdown
TODO: Add more content here.

FIXME: This section needs work.

XXX: Review this section.

WIP: Work in progress section.

<!-- TODO: Add content -->
```

### Correct

```markdown
This section documents the installation process in full.

<!-- Reviewed and complete -->
```

`BUG` is only flagged when it appears in comment-style context, such as `BUG:`,
`BUG(123):`, or after a code-comment marker like `// BUG`. Ordinary prose use of the
word is left alone:

```markdown
This kind of bug can be difficult to track down.
The bug fix was released yesterday.
```

Matches inside inline code spans (`` `TODO` ``) and, by default, inside fenced code
blocks are also skipped:

````markdown
Use `TODO` as a marker in your own commit messages.

```rust
// TODO: This is inside a code block and is not checked by default
```
````

## Configuration

```toml
[CONTENT001]
# Additional custom markers to detect, beyond (or instead of) the defaults.
markers = []

# Whether the built-in markers are checked: TODO, FIXME, XXX, HACK, WIP, and
# comment-style BUG. Default: true.
include_defaults = true

# Whether markers inside fenced code blocks are also checked. Default: false.
check_code_blocks = false
```

### `markers`

Add project-specific markers such as `REVIEW` or `NEEDSREVIEW`:

```toml
[CONTENT001]
markers = ["REVIEW"]
```

### `include_defaults`

Set to `false` to check only the custom `markers` list, ignoring the built-in
defaults:

```toml
[CONTENT001]
markers = ["REVIEW"]
include_defaults = false
```

### `check_code_blocks`

Set to `true` to also flag markers that appear inside fenced code blocks, such as
`// TODO` comments in a Rust example:

```toml
[CONTENT001]
check_code_blocks = true
```

## When to Disable

- Books that intentionally document their own outstanding work (a project's own
  TODO list rendered as a chapter)
- Draft content that is not yet meant for publication and is linted alongside
  finished chapters

## Rule Details

- **Rule ID**: CONTENT001
- **Aliases**: no-todo-comments
- **Category**: Content
- **Severity**: Warning
- **Auto-fix**: No

## Related Rules

- [CONTENT002](./content002.md) - Placeholder text such as lorem ipsum
- [CONTENT011](./content011.md) - Future tense describing what the software already does
