# MDBOOK007 - Include Validation

Include directives must point to existing files with valid syntax.

## Why This Rule Exists

mdBook's `\{{#include}}` directive embeds content from other files. Invalid
paths or syntax cause build failures or missing content.

## Examples

### Incorrect

```text
\{{#include missing-file.rs}}

\{{#include ../src/lib.rs:nonexistent_anchor}}

\{{include src/main.rs}}  <!-- Missing # -->
```

### Correct

```text
\{{#include ../src/lib.rs}}

\{{#include ../src/lib.rs:main_function}}

\{{#include ./snippets/example.rs:5:10}}
```

## Include Syntax

```text
<!-- Full file -->
\{{#include path/to/file.rs}}

<!-- Line range -->
\{{#include path/to/file.rs:5:10}}

<!-- From line to end -->
\{{#include path/to/file.rs:5:}}

<!-- From the start to a line -->
\{{#include path/to/file.rs::10}}

<!-- Named anchor -->
\{{#include path/to/file.rs:anchor_name}}
```

## Line Ranges and Anchors

MDBOOK007 reads the text after the path the same way mdBook does. If the part
before the first `:` is a number, or is empty, the spec is a line range.
Anything else is an anchor, named by that first part.

| Spec | Meaning |
| --- | --- |
| `7` | line 7 |
| `3:5` | lines 3 to 5 |
| `10:` | line 10 to the end of the file |
| `:10` | the start of the file to line 10 |
| `step2` | anchor `step2` |
| `abc:123` | anchor `abc` |

Anchor names may contain digits and may be of any length, so `a`, `v2` and
`example1` are all anchors. A mistyped line number such as `10abc` is also an
anchor to mdBook, so it is reported as a missing anchor.

## Anchors

mdBook finds an anchor wherever `ANCHOR:` appears on a line, whatever comment
syntax precedes it, and compares the name exactly. All of these declare anchor
`query`:

```text
// ANCHOR: query
# ANCHOR: query
-- ANCHOR: query
/* ANCHOR: query */
<!-- ANCHOR: query -->
```

`ANCHOR: query_all` does not declare `query`.

## Escaped Includes

A backslash immediately before an include directive makes mdBook render it as
literal text instead of reading the file. Books use this to show include
syntax to a reader. MDBOOK007 skips escaped directives, so the files and
anchors they name do not need to exist. For example, write
`\\{{#include missing-file.rs}}` to display the directive without reading
the file.

mdBook treats everything from an escaped directive to the last `}}` on the
same line as literal text, so a directive later on that line is not processed
either. A directive earlier on the line is processed normally.

## What Is Reported

- an include file that does not exist
- an anchor that is not declared in the included file
- a line number of 0
- a start line past the end of the file
- a start line after the end line
- an end line past the end of the file
- an end that is not a number, such as `10:abc`, which mdBook silently widens
  to the end of the file

mdBook builds without error in most of these cases, so the page is missing
content, or has the wrong content, with no warning.

## Configuration

This rule has no configuration options.

## When to Disable

- Files with includes resolved at a different build stage
- Templates with dynamic include paths

## Rule Details

- **Rule ID**: MDBOOK007
- **Aliases**: include-validation
- **Category**: MdBook
- **Severity**: Error
- **Auto-fix**: No

## Related Rules

- [MDBOOK008](./mdbook008.md) - Rustdoc include validation
- [MDBOOK012](./mdbook012.md) - Include line range validation
