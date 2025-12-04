# Image Rules

Rules for image formatting and accessibility.

## Rules in This Category

| Rule | Description | Auto-fix |
|------|-------------|----------|
| [MD045](./md045.md) | Images should have alt text | Yes |

## Overview

Image rules ensure that images are accessible and properly formatted.

### Why Alt Text Matters

Alt text (alternative text) serves multiple purposes:

- **Accessibility**: Screen readers use alt text to describe images
- **Fallback**: Displays when images fail to load
- **SEO**: Search engines use alt text to understand image content

### Best Practices

- Write descriptive alt text that conveys the image's purpose
- Keep alt text concise but informative
- For decorative images, use empty alt text `![](image.png)`
- Describe charts and diagrams with their key data points

### Example

```markdown
![Bar chart showing 50% increase in sales from Q1 to Q4](sales-chart.png)
```
