# Table Rules

Rules for formatting Markdown tables.

## Rules in This Category

| Rule | Description | Auto-fix |
|------|-------------|----------|
| [MD055](./md055.md) | Table pipe style consistency | Yes |
| [MD056](./md056.md) | Table column count | Yes |
| [MD058](./md058.md) | Tables surrounded by blank lines | Yes |

## Overview

Table rules ensure consistent and valid table formatting. Properly formatted
tables render correctly across all Markdown parsers.

### Common Issues

- Inconsistent pipe style (leading/trailing pipes)
- Rows with different numbers of columns
- Tables not separated from surrounding content

### Best Practices

- Use leading and trailing pipes for clarity
- Ensure all rows have the same number of columns
- Surround tables with blank lines
- Align columns for readable source (optional)

### Example Table

```markdown

| Header 1 | Header 2 | Header 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
| Cell 4   | Cell 5   | Cell 6   |

```
