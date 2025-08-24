# MDBOOK001 - Code Blocks Should Have Language Tags

{{#rustdoc_include ../../../../crates/mdbook-lint-rulesets/src/mdbook/mdbook001.rs:8:83}}

## Rule Details

- **Rule ID**: MDBOOK001
- **Category**: mdBook-specific
- **Severity**: Warning
- **Automatic Fix**: Not available (requires manual language identification)

## Why Language Tags Matter in mdBook

### Syntax Highlighting

mdBook uses language tags to apply syntax highlighting via highlight.js or similar libraries:

**Without language tag:**
````markdown
```
fn main() {
    println!("Hello, world!");
}
```
````

Renders as plain text with no highlighting.

**With language tag:**
````markdown
```rust
fn main() {
    println!("Hello, world!");
}
```
````

Renders with proper Rust syntax highlighting.

### mdBook-Specific Features

Language tags enable mdBook-specific features:

#### Rust Playground Integration

```rust
fn main() {
    println!("This code can be run in the Rust Playground!");
}
```

#### Hidden Lines in Rust Code

````markdown
```rust
# fn main() {
println!("Only this line is shown");
# }
```
````

#### Test Annotations

````markdown
```rust,ignore
// This code won't be tested
fn example() {}
```

```rust,no_run
// This code is compiled but not run
fn main() {
    loop {} // Would hang if run
}
```

```rust,should_panic
// This code is expected to panic
fn main() {
    panic!("This is expected!");
}
```
````

## Common Language Tags

### Programming Languages

| Language | Tag | Common Uses |
|----------|-----|-------------|
| Rust | `rust` | Primary language for mdBook documentation |
| JavaScript | `javascript` or `js` | Web examples, Node.js code |
| Python | `python` or `py` | Scripts, examples |
| Shell | `bash` or `sh` | Command-line examples |
| TOML | `toml` | Configuration files |
| JSON | `json` | Data structures, APIs |
| YAML | `yaml` or `yml` | Configuration, CI/CD |
| HTML | `html` | Web markup |
| CSS | `css` | Styling examples |
| SQL | `sql` | Database queries |

### Special Tags

| Tag | Purpose |
|-----|---------|
| `text` or `plain` | Plain text without highlighting |
| `console` | Terminal output with prompt highlighting |
| `diff` | Showing differences with +/- highlighting |
| `markdown` or `md` | Markdown source code |

## Examples by Use Case

### Configuration Files

**TOML (Cargo.toml):**
````markdown
```toml
[package]
name = "my-project"
version = "0.1.0"

[dependencies]
serde = "1.0"
```
````

**JSON (package.json):**
````markdown
```json
{
  "name": "my-project",
  "version": "1.0.0",
  "dependencies": {
    "react": "^18.0.0"
  }
}
```
````

### Command-Line Examples

**Shell commands:**
````markdown
```bash
# Install mdbook-lint
cargo install mdbook-lint

# Run the linter
mdbook-lint check
```
````

**Console output:**
````markdown
```console
$ cargo build
   Compiling my-project v0.1.0
    Finished dev [unoptimized] target(s) in 2.34s
```
````

### Showing Changes

**Diff format:**
````markdown
```diff
- Old line that was removed
+ New line that was added
  Unchanged line
```
````

### Multi-language Examples

**HTML with embedded CSS and JavaScript:**
````markdown
```html
<!DOCTYPE html>
<html>
<head>
    <style>
        body { font-family: sans-serif; }
    </style>
</head>
<body>
    <h1>Hello</h1>
    <script>
        console.log('Hello, world!');
    </script>
</body>
</html>
```
````

## Choosing the Right Language Tag

### Decision Tree

1. **Is it code?** → Use appropriate language tag
2. **Is it terminal output?** → Use `console`
3. **Is it a diff?** → Use `diff`
4. **Is it plain text?** → Use `text` or `plain`
5. **Is it data?** → Use format tag (`json`, `yaml`, `toml`)
6. **Not sure?** → Use `text` rather than no tag

### Language Detection Tips

Look for characteristic syntax:

- **Rust**: `fn`, `let`, `mut`, `impl`, `::`
- **Python**: `def`, `import`, `:` for blocks, no semicolons
- **JavaScript**: `function`, `const`, `=>`, `var`
- **Shell**: `$`, `#` for comments, command names
- **JSON**: `{`, `}`, `:`, quoted keys
- **TOML**: `[sections]`, `key = value`, `#` comments

## Edge Cases

### Mixed Language Blocks

For templates or mixed content, choose the primary language:

````markdown
```html
<!-- This is primarily HTML even though it contains CSS -->
<div style="color: red;">Content</div>
```
````

### Unknown or Custom Languages

For unsupported languages, use `text`:

````markdown
```text
CUSTOM_SYNTAX {
    nonstandard = syntax
}
```
````

### File Names as Context

Sometimes include the filename for context:

````markdown
```rust
// src/main.rs
fn main() {
    println!("Hello!");
}
```
````

## Impact on mdBook Features

### Search Indexing

Code blocks with language tags are better indexed for search.

### Syntax Theme Support

Language tags enable proper theme application:

- Light themes show appropriate colors
- Dark themes adjust for readability
- Contrast themes maintain accessibility

### Copy Button

mdBook's copy button works better with properly tagged code blocks.

### Line Numbers

Some themes show line numbers only for tagged code blocks:

````markdown
```rust,linenos
fn main() {
    println!("Line 1");
    println!("Line 2");
}
```
````

## Configuration

This rule has no configuration options. All code blocks should have language tags for optimal mdBook rendering.

## Related Rules

- [MD040](../standard/md040.md) - Fenced code blocks should have a language specified (standard rule)
- [MD046](../standard/md046.md) - Code block style
- [MD048](../standard/md048.md) - Code fence style

## References

- [mdBook - Code Blocks](https://rust-lang.github.io/mdBook/format/mdbook.html#code-blocks)
- [Highlight.js Languages](https://highlightjs.org/static/demo/)
- [CommonMark - Fenced Code Blocks](https://spec.commonmark.org/0.30/#fenced-code-blocks)