# MDBOOK017 - Hidden Code Prefix

Rust code blocks should use `#` to hide boilerplate from readers.

## Why This Rule Exists

In mdBook, lines in a Rust code block that start with `#` are hidden from the
rendered page but still compiled and run when the example is tested. Hiding
setup boilerplate such as `use` statements and `fn main() {}` wrappers keeps
the reader focused on the part of the example that actually matters, while
the example still compiles as real, tested code.

This rule looks for common boilerplate lines in fenced Rust blocks and flags
them when the block doesn't hide anything at all, on the assumption that an
author who hasn't used `#` anywhere in the block probably isn't hiding lines
on purpose.

## Examples

### Incorrect

````markdown
```rust
use std::collections::HashMap;

fn main() {
    let map: HashMap<i32, i32> = HashMap::new();
}
```
````

### Correct

````markdown
```rust
# use std::collections::HashMap;
# fn main() {
let map: HashMap<i32, i32> = HashMap::new();
# }
```
````

The hidden lines still compile, but a reader viewing the rendered book only
sees:

```rust
let map: HashMap<i32, i32> = HashMap::new();
```

### Already Aware of Hidden Lines

If the block hides at least one line, the rule assumes the author knows
about the feature and leaves the rest of the block alone, even if other
boilerplate in the same block is left visible:

````markdown
```rust
# use std::io;
fn main() {
    println!("Hello");
}
```
````

## Boilerplate Patterns

The rule flags a line when it starts with one of these patterns and the
surrounding block has no hidden (`#`-prefixed) lines:

- `use std::`
- `use crate::`
- `extern crate`
- `fn main() {` / `fn main(){`
- `pub fn main() {`
- `async fn main() {`
- `#![allow(`, `#![deny(`, `#![warn(`, `#![feature(`

Lines starting with `#[` (attributes like `#[derive(Debug)]`) and `#!`
(inner attributes) are not treated as hidden-line markers, since those are
ordinary visible Rust syntax rather than mdBook's hiding convention.

## Configuration

This rule has no configuration options.

## Rule Details

- **Rule ID**: MDBOOK017
- **Aliases**: hidden-code-prefix
- **Category**: MdBook
- **Severity**: Info
- **Stability**: Stable
- **Auto-fix**: No

## Related Rules

- [MDBOOK016](./mdbook016.md) - Rust code block attribute validation
- [MDBOOK008](./mdbook008.md) - Rustdoc include validation
