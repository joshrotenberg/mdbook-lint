# HTML Rules

Rules for inline HTML usage in Markdown.

## Rules in This Category

| Rule | Description | Auto-fix |
|------|-------------|----------|
| [MD033](./md033.md) | Inline HTML should be avoided | No |

## Overview

HTML rules control the use of raw HTML within Markdown documents. While
Markdown supports inline HTML, using it reduces portability and can
introduce security concerns.

### Why Avoid HTML

- **Portability**: Not all Markdown renderers support HTML
- **Security**: HTML can introduce XSS vulnerabilities
- **Maintainability**: Markdown is easier to read and maintain
- **Consistency**: Mixing HTML and Markdown creates inconsistent documents

### When HTML Is Acceptable

Some features require HTML:

- Collapsible sections (`<details>`)
- Keyboard shortcuts (`<kbd>`)
- Subscript/superscript (`<sub>`, `<sup>`)
- Complex layouts not possible in Markdown

### Configuration

Allow specific HTML elements while blocking others:

```toml
[MD033]
allowed_elements = ["details", "summary", "kbd", "br"]
```
