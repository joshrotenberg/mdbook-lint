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

`BUG` is handled by a separate check that looks for comment-style context. It is
flagged only when the word is followed by `:`, `(`, or `[` (whitespace may sit in
between), and only when it starts a line or follows whitespace or a comment marker
(`//`, `/*`, `#`):

```markdown
BUG: This needs fixing.
BUG(123): Tracked issue.
BUG[42] Bracket form.
// BUG: In a code comment.
```

The trailing punctuation is what triggers the match, not the comment marker. A bare
`// BUG` is not flagged, and neither is `BUG` alone on a line. The check is
case-insensitive and does not distinguish prose from anything else, so `there is a
bug: it crashes` is reported. These are not:

```markdown
This kind of bug can be difficult to track down.
The bug fix was released yesterday.
```

By default, matches inside inline code spans (`` `TODO` ``) and inside fenced code
blocks are skipped:

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

# Whether the built-in markers are checked: TODO, FIXME, XXX, HACK, WIP.
# Only takes effect when `markers` is non-empty. Default: true.
include_defaults = true

# Whether markers inside fenced code blocks and inline code spans are also
# checked. Default: false.
check_code_blocks = false
```

### `markers`

Add project-specific markers such as `REVIEW` or `NEEDSREVIEW`:

```toml
[CONTENT001]
markers = ["REVIEW"]
```

### `include_defaults`

Set to `false` alongside a non-empty `markers` list to check only the custom
markers:

```toml
[CONTENT001]
markers = ["REVIEW"]
include_defaults = false
```

With that configuration `TODO: ...` is no longer reported and `REVIEW: ...` is.

Two limits are worth knowing:

- `include_defaults = false` on its own does nothing. When `markers` is empty the
  rule falls back to the full default set, so every built-in marker is still
  checked.
- It does not govern `BUG`. The contextual `BUG` check runs unconditionally and
  cannot be turned off through this option or through `markers`.

### `check_code_blocks`

Set to `true` to also flag markers that appear inside fenced code blocks, such as
`// TODO` comments in a Rust example. The same setting governs inline code, so
`` `TODO` `` in a sentence is reported as well:

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
