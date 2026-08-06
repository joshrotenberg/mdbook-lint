# MDBOOK016 - Rust Code Block Attributes

Rust code blocks should use valid mdBook/rustdoc attributes.

## Why This Rule Exists

Rust code blocks accept a comma-separated list of attributes after the
`rust` language tag, such as `rust,ignore` or `rust,should_panic`. These
attributes control how mdBook and `rustdoc` treat the block when testing
the book. An unrecognized attribute is usually a typo that silently does
nothing instead of producing the intended behavior.

## Examples

### Incorrect

````markdown
```rust,invalid_attr
fn main() {}
```
````

````markdown
```rust,shouldpanic
fn main() { panic!(); }
```
````

### Correct

````markdown
```rust,ignore
fn main() {}
```
````

````markdown
```rust,should_panic
fn main() { panic!(); }
```
````

````markdown
```rust,no_run
fn main() {}
```
````

````markdown
```rust,ignore,editable
fn main() {}
```
````

When the attribute is a recognized typo, such as `shouldpanic`, `norun`, or
`compilefail`, the violation message suggests the correct spelling.

## Valid Attributes

- `ignore`, `noplayground`, `noplaypen`, `mdbook-runnable`, `editable`
- `hidelines` (and `hidelines=<prefix>`)
- `should_panic`, `no_run`, `compile_fail`
- `edition2015`, `edition2018`, `edition2021`, `edition2024`

Non-Rust code blocks (`python`, `javascript`, and so on) are not checked by
this rule. Only blocks tagged `rust` or `rs` are validated.

## Configuration

This rule has no configuration options.

## Rule Details

- **Rule ID**: MDBOOK016
- **Aliases**: rust-code-block-attributes
- **Category**: MdBook
- **Severity**: Warning
- **Auto-fix**: No

## Related Rules

- [MDBOOK017](./mdbook017.md) - Rust code block hidden lines
- [MD040](../standard/md040.md) - Fenced code blocks should have a language specified
