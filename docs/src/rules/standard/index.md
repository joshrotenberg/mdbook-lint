# Standard Markdown Rules

mdbook-lint implements 59 standard markdown linting rules based on the markdownlint specification. These rules help maintain consistent, readable, and properly formatted markdown documentation.

## Rule Categories

### [Heading Rules](./headings.md)

Rules for heading hierarchy, formatting, and style consistency.

### [List Rules](./lists.md)

Rules for list formatting, indentation, and marker consistency.

### [Whitespace Rules](./whitespace.md)

Rules for managing spaces, tabs, and blank lines.

### [Link Rules](./links.md)

Rules for URL formatting, link text, and reference links.

### [Code Rules](./code.md)

Rules for code blocks, inline code, and fencing style.

### [Emphasis Rules](./emphasis.md)

Rules for bold, italic, and other emphasis formatting.

## Complete Rule List

| Rule ID | Name | Description | Fix |
|---------|------|-------------|-----|
| MD001 | heading-increment | Heading levels should only increment by one level at a time | ❌ |
| MD002 | first-heading-h1 | First heading should be a top-level heading | ❌ |
| MD003 | heading-style | Heading style | ❌ |
| MD004 | ul-style | Unordered list style | ❌ |
| MD005 | list-indent | Inconsistent indentation for list items at the same level | ❌ |
| MD006 | ul-start-left | Consider starting lists at the beginning of the line | ❌ |
| MD007 | ul-indent | Unordered list indentation | ❌ |
| MD008 | no-bare-urls | Bare URLs should be wrapped in angle brackets | ❌ |
| MD009 | no-trailing-spaces | Trailing spaces | ✅ |
| MD010 | no-hard-tabs | Hard tabs | ✅ |
| MD011 | no-reversed-links | Reversed link syntax | ❌ |
| MD012 | no-multiple-blanks | Multiple consecutive blank lines | ✅ |
| MD013 | line-length | Line length | ❌ |
| MD014 | commands-show-output | Dollar signs used before commands without showing output | ❌ |
| MD015 | no-missing-space-closed-atx | No space after hash on closed atx style heading | ❌ |
| MD016 | no-reversed-heading-style | Heading levels should only increment | ❌ |
| MD017 | blanks-around-headings | Blank lines around headings | ❌ |
| MD018 | no-missing-space-atx | No space after hash on atx style heading | ✅ |
| MD019 | no-multiple-space-atx | Multiple spaces after hash on atx style heading | ✅ |
| MD020 | no-missing-space-closed-atx | No space inside hashes on closed atx style heading | ✅ |
| MD021 | no-multiple-space-closed-atx | Multiple spaces inside hashes on closed atx style heading | ✅ |
| MD022 | blanks-around-headings | Headings should be surrounded by blank lines | ❌ |
| MD023 | heading-start-left | Headings must start at the beginning of the line | ✅ |
| MD024 | no-duplicate-heading | Multiple headings with the same content | ❌ |
| MD025 | single-h1 | Multiple top-level headings in the same document | ❌ |
| MD026 | no-trailing-punctuation | Trailing punctuation in heading | ❌ |
| MD027 | no-multiple-space-blockquote | Multiple spaces after blockquote symbol | ✅ |
| MD028 | no-blanks-blockquote | Blank line inside blockquote | ❌ |
| MD029 | ol-prefix | Ordered list item prefix | ❌ |
| MD030 | list-marker-space | Spaces after list markers | ✅ |
| MD031 | blanks-around-fences | Fenced code blocks should be surrounded by blank lines | ❌ |
| MD032 | blanks-around-lists | Lists should be surrounded by blank lines | ❌ |
| MD033 | no-inline-html | Inline HTML | ❌ |
| MD034 | no-bare-urls | Bare URL used | ✅ |
| MD035 | hr-style | Horizontal rule style | ❌ |
| MD036 | no-emphasis-as-heading | Emphasis used instead of a heading | ❌ |
| MD037 | no-space-in-emphasis | Spaces inside emphasis markers | ❌ |
| MD038 | no-space-in-code | Spaces inside code span elements | ❌ |
| MD039 | no-space-in-links | Spaces inside link text | ❌ |
| MD040 | fenced-code-language | Fenced code blocks should have a language specified | ❌ |
| MD041 | first-line-h1 | First line in file should be a top-level heading | ❌ |
| MD042 | no-empty-links | No empty links | ❌ |
| MD043 | required-headings | Required heading structure | ❌ |
| MD044 | proper-names | Proper names should have correct capitalization | ❌ |
| MD045 | no-alt-text | Images should have alternate text | ❌ |
| MD046 | code-block-style | Code block style | ❌ |
| MD047 | single-trailing-newline | Files should end with a single newline character | ✅ |
| MD048 | code-fence-style | Code fence style | ❌ |
| MD049 | emphasis-style | Emphasis style should be consistent | ❌ |
| MD050 | strong-style | Strong style should be consistent | ❌ |
| MD051 | link-fragments | Link fragments should be valid | ❌ |
| MD052 | reference-links-images | Reference links and images should use a label that is defined | ❌ |
| MD053 | link-image-reference-definitions | Link and image reference definitions should be needed | ❌ |
| MD054 | link-image-style | Link and image style | ❌ |
| MD055 | table-pipe-style | Table pipe style | ❌ |
| MD056 | table-column-count | Table column count | ❌ |
| MD057 | table-rows | Table rows | ❌ |
| MD058 | blanks-around-tables | Tables should be surrounded by blank lines | ❌ |
| MD059 | table-alignment | Table alignment | ❌ |

Legend:

- ✅ Automatic fix available
- ❌ Manual fix required