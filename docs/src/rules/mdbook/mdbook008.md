# MDBOOK008 - Rustdoc Include Validation

Invalid `\{{#rustdoc_include}}` paths or syntax.

## Why This Rule Exists

The `\{{#rustdoc_include}}` directive is similar to `\{{#include}}` but hides
lines starting with `#` (used for rustdoc hidden lines). Invalid paths or
syntax cause build failures.

## Examples

### Incorrect

```text
\{{#rustdoc_include missing-file.rs}}

\{{#rustdoc_include ../src/lib.rs:bad_anchor}}

\{{rustdoc_include src/main.rs}}  <!-- Missing # -->
```

### Correct

```text
\{{#rustdoc_include ../src/lib.rs}}

\{{#rustdoc_include ../src/lib.rs:example}}

\{{#rustdoc_include ./snippets/demo.rs:5:20}}
```

## Rustdoc Include Syntax

```text
<!-- Full file, hiding # lines -->
\{{#rustdoc_include path/to/file.rs}}

<!-- Line range -->
\{{#rustdoc_include path/to/file.rs:5:10}}

<!-- Named anchor -->
\{{#rustdoc_include path/to/file.rs:anchor_name}}
```

## Hidden Lines

In the source file, lines starting with `#` are hidden:

```rust
# fn main() {
println!("This line is visible");
# }
```

Renders as just:

```rust
println!("This line is visible");
```

## Configuration

This rule has no configuration options.

## Rule Details

- **Rule ID**: MDBOOK008
- **Aliases**: rustdoc-include-validation
- **Category**: MdBook
- **Severity**: Error
- **Stability**: Experimental
- **Auto-fix**: No

## Related Rules

- [MDBOOK007](./mdbook007.md) - Include validation
- [MDBOOK012](./mdbook012.md) - Include line range validation
