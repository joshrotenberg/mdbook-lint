# CONTENT007 - Consistent Terminology

Terms should be used consistently throughout a document.

## Why This Rule Exists

Documentation that switches between variants of the same term, such as
"config" and "configuration", or "front-end" and "frontend", reads as
unpolished and can make readers wonder whether the two forms mean different
things. Settling on one spelling per document keeps prose predictable.

## Examples

### Incorrect

```markdown
# Settings

The config file is in the config directory.
Edit the configuration to change settings.
```

`config` is used twice and `configuration` once, so the less common variant
is flagged.

### Correct

```markdown
# Settings

The config file is in the config directory.
Edit the config to change settings.
```

Only one variant of the term is used throughout.

The rule also catches British/American spelling pairs and hyphenation
differences:

```markdown
<!-- Flagged: "colour" used once, "color" used twice -->
The color scheme is customizable.
You can change the colour of any element.
The colour picker is easy to use.
```

```markdown
<!-- Flagged: "frontend" used once, "front-end" used once -->
The frontend handles user interaction.
The front-end is built with React.
```

Matching is case-insensitive and respects word boundaries, so "reconfiguration"
does not match "config". Text inside fenced or inline code spans is ignored.

## Configuration

```toml
[CONTENT007]
# Groups of terms that should be used consistently. Replaces the built-in
# groups when provided.
term_groups = [
    ["config", "configuration"],
    ["setup", "set up", "set-up"],
    ["email", "e-mail"],
]

# Minimum number of times a less-common variant must appear before it is
# reported. Default: 1.
min_occurrences = 1
```

### `term_groups`

Each inner array lists variants that should not be mixed within a document.
The built-in groups cover common pairs such as `config`/`configuration`,
`login`/`log in`/`log-in`, `email`/`e-mail`, `frontend`/`front-end`,
`grey`/`gray`, `colour`/`color`, and similar. Supplying `term_groups` replaces
the defaults entirely rather than extending them.

### `min_occurrences`

Raises the bar before a variant is reported, so a single stray use of an
uncommon spelling does not trigger a violation. The default of `1` reports
every occurrence of a less-common variant once any inconsistency exists.

## When to Disable

- Documents that intentionally discuss both spellings, such as a style guide
  explaining the difference between "setup" (noun) and "set up" (verb)
- Books with contributors from different English locales where enforcing one
  spelling is not a priority

## Rule Details

- **Rule ID**: CONTENT007
- **Aliases**: consistent-terminology
- **Category**: Content
- **Severity**: Info
- **Auto-fix**: No

## Related Rules

- [CONTENT004](./content004.md) - Inconsistent heading capitalization
- [CONTENT010](./content010.md) - Non-descriptive link text
