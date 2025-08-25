# Code Rules

These rules ensure proper formatting of code blocks and inline code in markdown documents.

## Rules in This Category

- **[MD040](./md040.html)** - Fenced code blocks should have a language specified
- **MD014** - Dollar signs used before commands without showing output
- **MD031** - Fenced code blocks should be surrounded by blank lines
- **MD038** - Spaces inside code span elements
- **MD046** - Code block style
- **MD048** - Code fence style

## Why Code Rules Matter

Proper code formatting:
- Enables syntax highlighting for better readability
- Maintains consistency across code examples
- Improves copy-paste reliability
- Ensures proper rendering in different viewers
- Helps readers identify programming languages quickly

## Code Block Styles

### Fenced Code Blocks (Recommended)
````markdown
```javascript
function example() {
    return "Hello, world!";
}
```
````

### Indented Code Blocks
```markdown
    function example() {
        return "Hello, world!";
    }
```

### Inline Code
```markdown
Use the `console.log()` function to debug.
```

## Language Specifications

Common language tags for syntax highlighting:

| Language | Tags |
|----------|------|
| JavaScript | `js`, `javascript` |
| TypeScript | `ts`, `typescript` |
| Python | `py`, `python` |
| Rust | `rs`, `rust` |
| Shell | `sh`, `bash`, `shell` |
| JSON | `json` |
| YAML | `yml`, `yaml` |

## Quick Configuration

```toml
# .mdbook-lint.toml

# Configure MD040 - Require language tags
[rules.MD040]
allowed_languages = ["js", "python", "rust", "bash"]

# Configure MD046 - Code block style
[rules.MD046]
style = "fenced"  # Options: "fenced", "indented", "consistent"

# Configure MD048 - Code fence style
[rules.MD048]
style = "backtick"  # Options: "backtick", "tilde", "consistent"
```

## Best Practices

1. **Always specify language**: Enables syntax highlighting
2. **Use fenced blocks**: More flexible than indented blocks
3. **Surround with blank lines**: Improves readability
4. **Be consistent**: Use the same style throughout
5. **Escape special characters**: Use backslash when needed

### Shell Commands

```bash
# Good - shows command without prompt
npm install mdbook-lint

# Avoid - dollar sign without output
$ npm install mdbook-lint
```

## Related Categories

- [mdBook Rules](../mdbook/index.html) - mdBook-specific code requirements
- [Whitespace Rules](./whitespace.html) - General spacing rules