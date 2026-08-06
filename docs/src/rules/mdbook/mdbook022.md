# MDBOOK022 - Title Directive Near Top

`\{{#title}}` directive should appear near the top of the file.

## Why This Rule Exists

The `\{{#title}}` directive sets the page title shown in the browser tab.
mdBook processes it wherever it appears, but placing it far from the top of
the chapter makes it easy to miss and inconsistent with chapters that declare
their title up front. Keeping it within the first few lines makes the
chapter's title easy to find when skimming or editing the file.

## Examples

### Incorrect

```text
# Chapter

Paragraph 1.

Paragraph 2.

\{{#title Late Title}}

Content.
```

The directive is on line 7, past the default threshold of line 5.

### Correct

```text
\{{#title My Page Title}}

# Chapter Title

Content.
```

```text
# Chapter

Intro paragraph.

\{{#title My Title}}

More content.
```

The second example places the directive on line 5, which is still within the
default threshold. A chapter with no `\{{#title}}` directive at all is also
correct; this rule only checks the position of the first directive found.

## Configuration

```toml
[MDBOOK022]
# Highest line number at which a {{#title}} directive is still considered
# "near the top" of the file.
max_line = 5
```

`max-line` (kebab-case) is also accepted as an alias for `max_line`.

## Rule Details

- **Rule ID**: MDBOOK022
- **Aliases**: title-near-top
- **Category**: MdBook
- **Severity**: Warning
- **Stability**: Stable
- **Auto-fix**: No

## Related Rules

- [MDBOOK021](./mdbook021.md) - Single title directive per chapter
- [MDBOOK023](./mdbook023.md) - Title directive should match chapter title
