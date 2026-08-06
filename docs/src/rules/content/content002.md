# CONTENT002 - No Placeholder Text

Placeholder text should be replaced with actual content.

## Why This Rule Exists

Placeholder text like "Lorem ipsum", "TBD", or "coming soon" is a normal part
of drafting, but it should not survive into published documentation. This
rule flags a fixed set of common placeholder patterns so they can be caught
before a book ships.

## Examples

### Incorrect

```markdown
# Installation

Lorem ipsum dolor sit amet.
```

```markdown
# Configuration

This feature is TBD.
```

```markdown
# Roadmap

This section is coming soon.
```

```markdown
# API Reference

Insert content here.
```

### Correct

```markdown
# Installation

Run `cargo install mdbook-lint` to install the latest release.
```

```markdown
# Configuration

Set `output.max_width` to control line wrapping in generated pages.
```

Other patterns the rule flags: `TBA`, `TBC`, `under construction`, `work in
progress`, `N/A`, the literal word `placeholder`, `[draft]`, `[pending]`,
`foo bar baz`, `content goes here`, `your name here`, a line that is only
`XXX`, and a line that is only `...`. `example.com` is flagged too, unless
`allow_example_urls` is left at its default.

Matches inside fenced code blocks and inline code spans are skipped by
default, so a code sample showing `TBD` as a literal status value is not
flagged:

```markdown
Use `TBD` as the status value until the release date is set.
```

## Configuration

```toml
[CONTENT002]
# Whether to scan inside fenced code blocks. Default: false.
check_code_blocks = false

# Whether `example.com` is treated as an acceptable placeholder URL
# instead of being flagged. Default: true.
allow_example_urls = true
```

### `check_code_blocks`

With the default `false`, text inside fenced code blocks (and inline code
spans) is never checked, so example commands or sample output containing
words like `TBD` are left alone. Set it to `true` to also scan code blocks.

### `allow_example_urls`

With the default `true`, `example.com` is still reported when it appears in
prose (outside a code block); the option only affects whether it is skipped
inside code blocks once `check_code_blocks` is enabled.

## When to Disable

- Working drafts where placeholder markers are intentional and temporary
- Documentation that legitimately discusses these terms (for example, a
  writing-style guide that uses "lorem ipsum" as an example of placeholder
  text)

## Rule Details

- **Rule ID**: CONTENT002
- **Aliases**: no-placeholder-text
- **Category**: Content
- **Severity**: Warning
- **Auto-fix**: No

## Related Rules

- [CONTENT001](./content001.md) - TODO, FIXME and similar markers left in prose
- [CONTENT003](./content003.md) - Chapters too short to be worth a page
