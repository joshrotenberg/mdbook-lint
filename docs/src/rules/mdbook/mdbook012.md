# MDBOOK012 - Invalid Include Line Ranges

**Severity**: Error  
**Category**: mdBook-specific  
**Auto-fix**: Not available

## Rule Description

This rule validates line range specifications in `{{#include}}` and `{{#rustdoc_include}}` directives. It ensures line numbers are valid and the specified ranges exist in the target files.

## Why This Rule Exists

Valid line ranges are important because:

- Prevents build failures from invalid ranges
- Ensures included content is accurate
- Maintains documentation accuracy
- Helps identify outdated line references
- Prevents missing content in output

## Examples

### ❌ Incorrect (violates rule)

```markdown
<!-- Invalid range syntax -->
\{{#include ./file.md:1-10}}      <!-- Should use colons -->
\{{#include ./file.md:10:5}}      <!-- End before start -->
\{{#include ./file.md:0:10}}      <!-- Line numbers start at 1 -->
\{{#include ./file.md:-5:10}}     <!-- Negative line number -->

<!-- Out of bounds (file has 50 lines) -->
\{{#include ./file.md:45:60}}     <!-- End exceeds file length -->
\{{#include ./file.md:100}}       <!-- Line doesn't exist -->

<!-- Invalid format -->
\{{#include ./file.md:a:b}}       <!-- Non-numeric -->
\{{#include ./file.md::}}         <!-- Empty range -->
```

### ✅ Correct

```markdown
<!-- Include specific lines -->
\{{#include ./file.md:1:10}}      <!-- Lines 1-10 -->
\{{#include ./file.md:5}}         <!-- Only line 5 -->

<!-- Open-ended ranges -->
\{{#include ./file.md:10:}}       <!-- From line 10 to end -->
\{{#include ./file.md::20}}       <!-- From start to line 20 -->

<!-- Using anchors instead of lines -->
\{{#include ./file.md:example}}   <!-- More stable than line numbers -->
```

## Line Range Syntax

### Syntax Options

| Syntax | Description | Example |
|--------|-------------|---------|
| `:start:end` | Lines from start to end | `:1:10` |
| `:line` | Single line | `:42` |
| `:start:` | From start to end of file | `:10:` |
| `::end` | From beginning to end | `::25` |
| `:anchor` | Include anchor section | `:my_example` |

### Examples 2

```markdown
<!-- Include lines 5-10 -->
\{{#include ./example.rs:5:10}}

<!-- Include from line 20 to end -->
\{{#include ./example.rs:20:}}

<!-- Include first 15 lines -->
\{{#include ./example.rs::15}}

<!-- Include single line -->
\{{#include ./example.rs:7}}
```

## Configuration

```toml
[MDBOOK012]
validate_bounds = true    # Check if line numbers exist (default: true)
warn_large_ranges = true  # Warn for ranges > 100 lines (default: true)
max_range_size = 100      # Maximum lines in a range (default: 100)
prefer_anchors = true     # Suggest anchors over line numbers (default: true)
```

## Common Issues and Solutions

### Issue: File Changes Break Ranges

```markdown
<!-- Original: function at lines 10-15 -->
\{{#include ./code.rs:10:15}}

<!-- After refactoring: function moved to lines 25-30 -->
<!-- BROKEN: Still references old location -->
\{{#include ./code.rs:10:15}}

<!-- Solution: Use anchors -->
\{{#include ./code.rs:my_function}}
```

### Issue: Off-by-One Errors

```markdown
<!-- Want to include lines with function (lines 5-8) -->

<!-- Wrong: Missing first line -->
\{{#include ./code.rs:6:8}}

<!-- Correct: Include all lines -->
\{{#include ./code.rs:5:8}}
```

### Issue: Including Too Much

```markdown
<!-- Bad: Including entire file when only need part -->
\{{#include ./large-file.md:1:500}}

<!-- Better: Include specific section -->
\{{#include ./large-file.md:45:67}}

<!-- Best: Use anchor -->
\{{#include ./large-file.md:relevant_section}}
```

## Best Practices

1. **Prefer anchors over line numbers**: More stable across edits
2. **Keep ranges small**: Include only what's necessary
3. **Document ranges**: Comment what's being included
4. **Update after refactoring**: Check includes after moving code
5. **Test includes**: Verify correct content is included

### Using Anchors Instead

```rust
// code.rs
// ANCHOR: database_connection
fn connect_to_database() -> Result<Connection, Error> {
    let url = env::var("DATABASE_URL")?;
    Connection::new(&url)
}
// ANCHOR_END: database_connection
```

```markdown
<!-- More stable than line numbers -->
\{{#include ./code.rs:database_connection}}
```

### Documenting Includes

```markdown
<!-- Include the main function (lines 15-25) -->
\{{#include ./example.rs:15:25}}

<!-- Include error handling example -->
\{{#include ./errors.rs:error_handling}}
```

## Line Counting Rules

1. **Line numbers start at 1**: First line is line 1, not 0
2. **Inclusive ranges**: Both start and end lines are included
3. **Empty lines count**: Blank lines are counted in numbering
4. **Comments count**: Comment lines are included in count

### Example File Numbering

```rust
// Line 1: Comment
// Line 2: Another comment
                          // Line 3: Empty line
fn main() {               // Line 4
    println!("Hello");    // Line 5
}                         // Line 6
```

## When to Disable

Consider disabling this rule if:

- Your files are generated and line numbers are stable
- You're migrating content with many includes
- You use a different include mechanism
- Your build process validates includes separately

### Disable in Config

```toml
# .mdbook-lint.toml
disabled_rules = ["MDBOOK012"]
```

### Disable Inline

```markdown
<!-- mdbook-lint-disable MDBOOK012 -->
\{{#include ./generated.md:1:1000}}
<!-- mdbook-lint-enable MDBOOK012 -->
```

## Debugging Line Ranges

### View File with Line Numbers

```bash
# Show file with line numbers
cat -n file.md

# Show specific lines
sed -n '10,20p' file.md

# Count total lines
wc -l file.md
```

### Test Include Output

```bash
# Build and check included content
mdbook build
grep -A5 -B5 "included content" book/chapter.html
```

## Error Messages

| Error | Solution |
|-------|----------|
| "Line number out of range" | Check file length |
| "Invalid range syntax" | Use correct format `:start:end` |
| "End before start" | Ensure start < end |
| "File not found" | Verify file path |

## Related Rules

- [MDBOOK007](./mdbook007.html) - Include directive validation
- [MDBOOK008](./mdbook008.html) - Rustdoc include validation

## References

- [mdBook - Including Files](https://rust-lang.github.io/mdBook/format/markdown.html#including-files)
- [mdBook - Including Portions](https://rust-lang.github.io/mdBook/format/markdown.html#including-portions-of-files)
