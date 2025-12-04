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

<!-- Named anchor -->
\{{#include path/to/file.rs:anchor_name}}
```

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
