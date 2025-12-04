# MDBOOK011 - Template Validation

Invalid `\{{#template}}` syntax.

## Why This Rule Exists

The `\{{#template}}` directive expands templates with variable substitution.
Invalid syntax or missing variables cause build failures.

## Examples

### Incorrect

```text
\{{#template missing-template.md}}

\{{#template ./template.md var1=value}}  <!-- Missing closing -->

\{{template ./template.md}}  <!-- Missing # -->
```

### Correct

```text
\{{#template ./templates/note.md}}

\{{#template ./templates/warning.md title="Important" content="Read carefully"}}
```

## Template Syntax

```text
<!-- Basic template -->
\{{#template path/to/template.md}}

<!-- With variables -->
\{{#template path/to/template.md var1="value1" var2="value2"}}
```

### Template File

```text
<!-- templates/note.md -->
> **\{{title}}**
>
> \{{content}}
```

### Usage

```text
\{{#template templates/note.md title="Note" content="This is important."}}
```

## Configuration

This rule has no configuration options.

## Rule Details

- **Rule ID**: MDBOOK011
- **Aliases**: template-validation
- **Category**: MdBook
- **Severity**: Warning
- **Stability**: Experimental
- **Auto-fix**: No

## Related Rules

- [MDBOOK007](./mdbook007.md) - Include validation
- [MDBOOK010](./mdbook010.md) - Preprocessor validation
