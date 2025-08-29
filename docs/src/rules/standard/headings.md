# Heading Rules

Heading rules ensure proper document structure, hierarchy, and formatting for markdown headings.

## Rules in This Category

| Rule | Description | Fix |
|------|-------------|-----|
| [MD001](./md001.md) | Heading levels should only increment by one level at a time | ❌ |
| [MD002](./md002.md) | First heading should be a top-level heading | ❌ |
| [MD003](./md003.md) | Heading style (ATX vs Setext) | ❌ |
| [MD018](./md018.md) | No space after hash on ATX style heading | ✅ |
| [MD019](./md019.md) | Multiple spaces after hash on ATX style heading | ✅ |
| [MD020](./md020.md) | No space inside hashes on closed ATX style heading | ✅ |
| [MD021](./md021.md) | Multiple spaces inside hashes on closed ATX style heading | ✅ |
| [MD022](./md022.md) | Headings should be surrounded by blank lines | ❌ |
| [MD023](./md023.md) | Headings must start at the beginning of the line | ✅ |
| [MD024](./md024.md) | Multiple headings with the same content | ❌ |
| [MD025](./md025.md) | Multiple top-level headings in the same document | ❌ |
| [MD026](./md026.md) | Trailing punctuation in heading | ❌ |
| [MD041](./md041.md) | First line in file should be a top-level heading | ❌ |

## Best Practices

### Document Structure

A well-structured document follows these heading principles:

1. **Start with H1**: Documents should begin with a single H1 heading
2. **Sequential Levels**: Never skip heading levels (H1 → H3 is wrong)
3. **Logical Hierarchy**: Use headings to create a document outline
4. **Consistent Style**: Use either ATX (`#`) or Setext style consistently

### ATX vs Setext Headings

**ATX Style** (Recommended):
```markdown
# Heading 1
## Heading 2
### Heading 3
```

**Setext Style** (Limited to H1 and H2):
```markdown
Heading 1
=========

Heading 2
---------
```

### Closed ATX Headings

Some prefer closed ATX headings for symmetry:

```markdown
# Heading 1 #
## Heading 2 ##
### Heading 3 ###
```

Rules MD020 and MD021 ensure proper formatting of closed headings.

## Common Issues and Solutions

### Issue: Inconsistent Heading Hierarchy

**Problem**: Jumping between heading levels disrupts document flow.

```markdown
# Main Title
### Subsection (skips H2)
## Back to H2
##### Deep section (skips H3 and H4)
```

**Solution**: Maintain sequential heading levels.

```markdown
# Main Title
## Section
### Subsection
## Another Section
### Subsection
#### Deeper Content
##### Deepest Content
```

### Issue: Multiple H1 Headings

**Problem**: Multiple top-level headings confuse document structure.

```markdown
# First Title
Content...
# Second Title
More content...
```

**Solution**: Use a single H1 with H2s for major sections.

```markdown
# Document Title
## First Section
Content...
## Second Section
More content...
```

### Issue: Indented Headings

**Problem**: Headings with leading spaces may not render correctly.

```markdown
    # This might not be a heading
  ## This is problematic
```

**Solution**: Start headings at the beginning of the line.

```markdown
# Proper Heading
## Another Proper Heading
```

## Accessibility Considerations

Proper heading structure is crucial for accessibility:

- **Screen Readers**: Use headings to navigate and understand document structure
- **Keyboard Navigation**: Many tools allow jumping between headings
- **Document Outline**: Assistive technologies generate outlines from headings
- **WCAG Compliance**: Proper heading hierarchy is part of WCAG 2.1 guidelines

## Integration with mdBook

mdBook relies heavily on proper heading structure:

1. **Table of Contents**: Generated from heading hierarchy
2. **Search Index**: Headings are weighted in search results
3. **Navigation**: Sidebar navigation reflects heading structure
4. **Anchors**: Automatic anchor generation for deep linking

## Configuration Examples

### Enforce ATX Style Only

```toml
[MD003]
style = "atx"
```

### Allow Trailing Punctuation

```toml
[MD026]
enabled = false
```

### Require Document to Start with H1

```toml
[MD041]
level = 1
front_matter_title = false
```