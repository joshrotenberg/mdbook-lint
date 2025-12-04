# MDBOOK012 - Include Line Range Validation

Broken `\{{#include}}` line ranges.

## Why This Rule Exists

Include directives with line ranges must reference valid line numbers. Ranges
that exceed the file length or have invalid syntax cause build failures or
unexpected content.

## Examples

### Incorrect

```text
<!-- File has only 50 lines -->
\{{#include ../src/lib.rs:100:150}}

<!-- Invalid range (end before start) -->
\{{#include ../src/lib.rs:20:10}}

<!-- Non-numeric range -->
\{{#include ../src/lib.rs:start:end}}
```

### Correct

```text
\{{#include ../src/lib.rs:1:10}}

\{{#include ../src/lib.rs:5:}}

\{{#include ../src/lib.rs::20}}
```

## Line Range Syntax

```text
<!-- Lines 5 through 10 -->
\{{#include file.rs:5:10}}

<!-- Line 5 to end of file -->
\{{#include file.rs:5:}}

<!-- Start of file through line 10 -->
\{{#include file.rs::10}}

<!-- Single line (line 5 only) -->
\{{#include file.rs:5:5}}
```

## Configuration

This rule has no configuration options.

## Rule Details

- **Rule ID**: MDBOOK012
- **Aliases**: include-line-range-validation
- **Category**: MdBook
- **Severity**: Error
- **Stability**: Experimental
- **Auto-fix**: No

## Related Rules

- [MDBOOK007](./mdbook007.md) - Include validation
- [MDBOOK008](./mdbook008.md) - Rustdoc include validation
