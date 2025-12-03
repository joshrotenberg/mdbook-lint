# List Rules

List rules ensure consistent formatting, indentation, and structure for both ordered and unordered lists.

## Rules in This Category

| Rule | Description | Fix |
|------|-------------|-----|
| [MD004](./md004.md) | Unordered list style (consistent markers) | ❌ |
| [MD005](./md005.md) | Inconsistent indentation for list items at the same level | ❌ |
| [MD006](./md006.md) | Consider starting lists at the beginning of the line | ❌ |
| [MD007](./md007.md) | Unordered list indentation | ❌ |
| [MD029](./md029.md) | Ordered list item prefix | ❌ |
| [MD030](./md030.md) | Spaces after list markers | ✅ |
| [MD031](./md031.md) | Fenced code blocks should be surrounded by blank lines | ❌ |
| [MD032](./md032.md) | Lists should be surrounded by blank lines | ❌ |

## List Basics

### Unordered Lists

Markdown supports three markers for unordered lists:

```markdown
* Item with asterisk
- Item with dash
+ Item with plus
```

All render the same, but consistency is important.

### Ordered Lists

```markdown
1. First item
2. Second item
3. Third item
```

Or with lazy numbering:

```markdown
1. First item
1. Second item
1. Third item
```

## Best Practices

### Consistent Markers

Pick one unordered list marker and stick with it:

**Good:**

```markdown
* First item
* Second item
  * Nested item
  * Another nested
* Third item
```

**Bad:**

```markdown
* First item
- Second item
  + Nested item
  * Another nested
+ Third item
```

### Proper Indentation

Use consistent indentation for nested lists:

**2-space indentation:**

```markdown
* Parent item
  * Child item
    * Grandchild item
  * Another child
* Another parent
```

**4-space indentation:**

```markdown
* Parent item
    * Child item
        * Grandchild item
    * Another child
* Another parent
```

### Spacing After Markers

Maintain consistent spacing after list markers:

**Good (single space):**

```markdown
* Item one
* Item two
1. First item
2. Second item
```

**Bad (inconsistent):**

```markdown
*Item one
*  Item two
1.First item
2.  Second item
```

## Complex List Structures

### Multi-line List Items

For list items with multiple paragraphs:

```markdown
1. First item with multiple paragraphs.

   This is still part of the first item. Note the blank line above
   and the indentation.

2. Second item.

   * Nested list in second item
   * Another nested item

3. Third item.
```

### Lists with Code Blocks

Proper indentation for code blocks in lists:

```markdown
1. Install the package:

   ```bash
   npm install mdbook-lint
   ```

1. Configure the linter:

   ```json
   {
     "rules": {
       "MD009": true
     }
   }
   ```

2. Run the linter.

```

### Task Lists

GitHub Flavored Markdown task lists:

```markdown
- [x] Completed task
- [ ] Incomplete task
- [ ] Another todo
  - [x] Completed subtask
  - [ ] Incomplete subtask
```

## Common Issues and Solutions

### Issue: Inconsistent List Indentation

**Problem:**

```markdown
* Item 1
  * Nested with 2 spaces
    * Deep nested with 4 spaces
 * Wrong indentation
   * More inconsistency
```

**Solution:**

```markdown
* Item 1
  * Nested with 2 spaces
  * Consistent 2-space indent
  * All items aligned
    * Deeper nesting maintains pattern
```

### Issue: Missing Blank Lines Around Lists

**Problem:**

```markdown
Some paragraph text
* List starts immediately
* No separation
Paragraph continues here
```

**Solution:**

```markdown
Some paragraph text

* List has blank line before
* Proper separation

Paragraph has blank line after list
```

### Issue: Lazy Numbering Problems

**Problem with lazy numbering:**

```markdown
1. First item
1. Second item
5. Oops, wrong number
1. Fourth item
```

**Solution 1 (sequential):**

```markdown
1. First item
2. Second item
3. Third item
4. Fourth item
```

**Solution 2 (all ones):**

```markdown
1. First item
1. Second item
1. Third item
1. Fourth item
```

## Accessibility Considerations

Proper list formatting improves accessibility:

1. **Screen Readers**: Announce list structure and item count
2. **Navigation**: Users can skip between lists
3. **Context**: Proper nesting conveys relationships
4. **Semantics**: Lists convey meaning beyond visual formatting

## mdBook-Specific Considerations

In mdBook projects:

1. **Table of Contents**: Lists in SUMMARY.md define book structure
2. **Navigation**: Nested lists create hierarchical navigation
3. **Rendering**: List formatting affects HTML output
4. **Search**: List items are indexed for search

## Configuration Examples

### Enforce Consistent Unordered List Style

```toml
[MD004]
style = "asterisk"  # or "dash" or "plus"
```

### Set List Indentation

```toml
[MD007]
indent = 2  # or 4, or any consistent value
```

### Configure Ordered List Style

```toml
[MD029]
style = "ordered"  # or "one" for all 1s
```

### Spaces After List Markers

```toml
[MD030]
ul_single = 1  # Spaces after unordered list marker
ol_single = 1  # Spaces after ordered list marker
ul_multi = 1   # Spaces after marker for multi-line items
ol_multi = 1   # Spaces after marker for multi-line items
```

## Related Rules

- [MD013](./md013.md) - Line length (affects long list items)
- [MD022](./md022.md) - Blank lines (around list blocks)
- [MD031](./md031.md) - Code blocks in lists
- [MD032](./md032.md) - Blank lines around lists

## References

- [CommonMark Spec - Lists](https://spec.commonmark.org/0.30/#lists)
- [GitHub Flavored Markdown - Task Lists](https://github.github.com/gfm/#task-list-items-extension-)
- [mdBook - SUMMARY.md Format](https://rust-lang.github.io/mdBook/format/summary.html)
