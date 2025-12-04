# MDBOOK009 - Playground Validation

Invalid `\{{#playground}}` configuration.

## Why This Rule Exists

The `\{{#playground}}` directive creates interactive Rust code examples. Invalid
paths or configuration cause build failures or non-functional playgrounds.

## Examples

### Incorrect

```text
\{{#playground missing-file.rs}}

\{{#playground ../src/example.rs invalid_option}}

\{{playground src/demo.rs}}  <!-- Missing # -->
```

### Correct

```text
\{{#playground ../src/example.rs}}

\{{#playground ../src/example.rs editable}}

\{{#playground ../src/example.rs editable hide_lines=1-3}}
```

## Playground Options

```text
<!-- Basic playground -->
\{{#playground path/to/file.rs}}

<!-- Editable playground -->
\{{#playground path/to/file.rs editable}}

<!-- Hide specific lines -->
\{{#playground path/to/file.rs hide_lines=1-3}}

<!-- Multiple options -->
\{{#playground path/to/file.rs editable no_run}}
```

## Available Options

| Option | Description |
|--------|-------------|
| `editable` | Allow users to edit the code |
| `no_run` | Show code but disable running |
| `ignore` | Don't test this code |
| `hide_lines` | Hide specific line ranges |

## Configuration

This rule has no configuration options.

## Rule Details

- **Rule ID**: MDBOOK009
- **Aliases**: playground-validation
- **Category**: MdBook
- **Severity**: Warning
- **Stability**: Experimental
- **Auto-fix**: No

## Related Rules

- [MDBOOK001](./mdbook001.md) - Code block language tags
- [MDBOOK007](./mdbook007.md) - Include validation
