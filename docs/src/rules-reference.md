# Rules Reference

This page provides a comprehensive reference for all **78 linting rules** available in mdbook-lint.

## Quick Navigation

- [Complete Rule List](#complete-rule-list) - All 78 rules at a glance
- [Standard Markdown Rules (MD001-MD060)](#standard-markdown-rules) - 55 standard rules
- [mdBook-Specific Rules](#mdbook-specific-rules) - 18 mdBook rules
- [Content Rules](#content-rules) - 5 content rules
- [Auto-Fix Rules](#auto-fix-rules) - Rules with automatic fixes
- [Rule Configuration](#rule-configuration) - How to customize rules

## Complete Rule List

### Standard Rules (55 rules)

| Rule | Name | Auto-fix | Category |
|------|------|----------|----------|
| [MD001](./rules/standard/md001.html) | Heading increment | ✓ | Structure |
| [MD002](#md002) | First heading should be top level | ✓ | Structure |
| [MD003](#md003) | Heading style consistency | ✓ | Style |
| [MD004](#md004) | Unordered list style | ✓ | Lists |
| [MD005](#md005) | List item indentation consistency | ✓ | Lists |
| [MD006](#md006) | Start lists at beginning of line | ✓ | Lists |
| [MD007](#md007) | Unordered list indentation | ✓ | Lists |
| [MD009](./rules/standard/md009.html) | No trailing spaces | ✓ | Whitespace |
| [MD010](./rules/standard/md010.html) | Hard tabs | ✓ | Whitespace |
| [MD011](#md011) | Reversed link syntax | ✓ | Links |
| [MD012](./rules/standard/md012.html) | Multiple consecutive blank lines | ✓ | Whitespace |
| [MD013](./rules/standard/md013.html) | Line length | | Style |
| [MD014](#md014) | Dollar signs in shell code | ✓ | Code |
| [MD018](./rules/standard/md018.html) | No space after hash | ✓ | Headings |
| [MD019](./rules/standard/md019.html) | Multiple spaces after hash | ✓ | Headings |
| [MD020](./rules/standard/md020.html) | No space in closed headings | ✓ | Headings |
| [MD021](./rules/standard/md021.html) | Multiple spaces in closed headings | ✓ | Headings |
| [MD022](#md022) | Headings surrounded by blank lines | ✓ | Headings |
| [MD023](./rules/standard/md023.html) | Headings start at beginning | ✓ | Headings |
| [MD024](#md024) | Multiple headings same content | ✓ | Headings |
| [MD025](#md025) | Multiple top-level headings | ✓ | Headings |
| [MD026](#md026) | Trailing punctuation in headings | ✓ | Headings |
| [MD027](./rules/standard/md027.html) | Multiple spaces after blockquote | ✓ | Blockquotes |
| [MD028](#md028) | Blank line inside blockquote | ✓ | Blockquotes |
| [MD029](#md029) | Ordered list item prefix | ✓ | Lists |
| [MD030](./rules/standard/md030.html) | Spaces after list markers | ✓ | Lists |
| [MD031](#md031) | Fenced code blocks surrounded | ✓ | Code |
| [MD032](#md032) | Lists surrounded by blank lines | ✓ | Lists |
| [MD033](#md033) | Inline HTML | | HTML |
| [MD034](./rules/standard/md034.html) | Bare URL used | ✓ | Links |
| [MD035](#md035) | Horizontal rule style | ✓ | Style |
| [MD036](#md036) | Emphasis instead of heading | | Emphasis |
| [MD037](#md037) | Spaces inside emphasis markers | ✓ | Emphasis |
| [MD038](#md038) | Spaces inside code spans | ✓ | Code |
| [MD039](#md039) | Spaces inside link text | ✓ | Links |
| [MD040](./rules/standard/md040.html) | Fenced code blocks language | | Code |
| [MD041](#md041) | First line top-level heading | | Structure |
| [MD042](#md042) | No empty links | | Links |
| [MD043](#md043) | Required heading structure | | Structure |
| [MD044](#md044) | Proper names capitalization | | Style |
| [MD045](#md045) | Images should have alt text | ✓ | Images |
| [MD046](#md046) | Code block style | ✓ | Code |
| [MD047](./rules/standard/md047.html) | Files end with newline | ✓ | Whitespace |
| [MD048](#md048) | Code fence style | ✓ | Code |
| [MD049](#md049) | Emphasis style consistency | ✓ | Emphasis |
| [MD050](#md050) | Strong style consistency | ✓ | Emphasis |
| [MD051](#md051) | Link fragments | | Links |
| [MD052](#md052) | Reference links and images | | Links |
| [MD053](#md053) | Link and image reference definitions | | Links |
| [MD054](#md054) | Link and image style | | Links |
| [MD055](#md055) | Table pipe style | ✓ | Tables |
| [MD056](#md056) | Table column count | ✓ | Tables |
| [MD058](#md058) | Tables surrounded by blank lines | ✓ | Tables |
| [MD059](#md059) | Link and image reference style | | Links |
| [MD060](#md060) | Table column style | | Tables |

### mdBook Rules (18 rules)

| Rule | Name | Auto-fix | Purpose |
|------|------|----------|---------|
| [MDBOOK001](./rules/mdbook/mdbook001.html) | Code blocks should have language tags | | Syntax highlighting |
| [MDBOOK002](./rules/mdbook/mdbook002.html) | Internal link validation | | Link integrity |
| [MDBOOK003](./rules/mdbook/mdbook003.html) | SUMMARY.md structure | | Book structure |
| [MDBOOK004](./rules/mdbook/mdbook004.html) | Unique chapter titles | | Book structure |
| [MDBOOK005](./rules/mdbook/mdbook005.html) | Orphaned files detection | | File management |
| [MDBOOK006](#mdbook006) | Cross-reference validation | | Navigation |
| [MDBOOK007](#mdbook007) | File include syntax | | Includes |
| [MDBOOK008](#mdbook008) | Rustdoc include validation | | Documentation |
| [MDBOOK009](#mdbook009) | Playground directive syntax | | Code examples |
| [MDBOOK010](#mdbook010) | Preprocessor configuration | | Configuration |
| [MDBOOK011](#mdbook011) | Template include syntax | | Templates |
| [MDBOOK012](#mdbook012) | File include ranges | | Includes |
| [MDBOOK016](#mdbook016) | Valid Rust code block attributes | | Rust code |
| [MDBOOK017](#mdbook017) | Hidden Rust boilerplate | | Rust code |
| [MDBOOK021](#mdbook021) | Single title directive | | Directives |
| [MDBOOK022](#mdbook022) | Title directive placement | | Directives |
| [MDBOOK023](#mdbook023) | Chapter title consistency | | Book structure |
| [MDBOOK025](#mdbook025) | SUMMARY.md heading structure | | Table of contents |

### Content Rules (5 rules)

| Rule | Name | Auto-fix | Purpose |
|------|------|----------|---------|
| [CONTENT001](#content001) | No TODO/FIXME comments | | Content quality |
| [CONTENT002](#content002) | No placeholder text | | Content quality |
| [CONTENT003](#content003) | Minimum chapter length | | Content quality |
| [CONTENT004](#content004) | Heading capitalization | | Style consistency |
| [CONTENT005](#content005) | Intro before subheading | | Content structure |

*✓ indicates automatic fix support*

## Standard Markdown Rules

### Heading Rules (MD001-MD003, MD018-MD026)

Control heading structure, style, and formatting:

- **[MD001](./rules/standard/md001.html)** - Ensures sequential heading levels (H1→H2→H3)
- **MD002** - First heading should be top-level
- **MD003** - Consistent heading style (ATX vs Setext)
- **MD018** ✓ - Space required after `#` in headings
- **MD019** ✓ - Single space after `#` in headings
- **MD020** ✓ - No spaces inside `### Heading ###`
- **MD021** ✓ - Single space inside `### Heading ###`
- **MD022** - Headings surrounded by blank lines
- **MD023** ✓ - Headings start at line beginning
- **MD024** - No duplicate heading content
- **MD025** - Single top-level heading per document
- **MD026** - No trailing punctuation in headings

### List Rules (MD004-MD007, MD029-MD032)

Ensure consistent list formatting:

- **MD004** - Consistent unordered list markers (`*`, `-`, `+`)
- **MD005** - Consistent indentation within list levels
- **MD006** - Lists start at line beginning
- **MD007** - Proper nested list indentation
- **MD029** - Ordered list number style (1. vs 1))
- **MD030** ✓ - Spaces after list markers
- **MD031** - Fenced code blocks surrounded by blanks
- **MD032** - Lists surrounded by blank lines

### Whitespace Rules (MD009-MD012, MD027-MD028, MD047)

Control spacing and blank lines:

- **[MD009](./rules/standard/md009.html)** ✓ - Remove trailing spaces
- **MD010** ✓ - Hard tabs → spaces
- **MD012** ✓ - Limit consecutive blank lines
- **MD027** ✓ - Single space after blockquote `>`
- **MD028** - No blank lines inside blockquotes
- **MD047** ✓ - Files end with single newline

### Code Rules (MD014, MD031, MD038, MD040, MD046-MD048)

Validate code blocks and inline code:

- **MD014** - No `$` prompts in shell code
- **MD031** - Code blocks surrounded by blanks
- **MD038** - No spaces inside `code spans`
- **MD040** - Language tags for fenced code
- **MD046** - Consistent code block style
- **MD048** - Consistent code fence style

### Link and Image Rules (MD011, MD034, MD039, MD042, MD045, MD051-MD054, MD059)

Ensure proper links and images:

- **MD011** - Fix reversed link syntax `](link)[text`
- **MD034** ✓ - Use `[text](url)` not bare URLs
- **MD039** - No spaces in `[text](url)`

- **MD042** - No empty links `[]()`
- **MD045** - Images need alt text
- **MD051-MD054, MD059** - Reference link consistency

## mdBook-Specific Rules

### Content Structure

- **[MDBOOK001](./rules/mdbook/mdbook001.html)** - Code blocks need language tags for syntax highlighting
- **MDBOOK003** - SUMMARY.md follows mdBook conventions
- **MDBOOK025** - Proper heading hierarchy in SUMMARY.md

### Link Validation  

- **MDBOOK002** - Internal links point to valid files/anchors
- **MDBOOK006** - Cross-references between chapters work

### File Management

- **MDBOOK005** - Detect orphaned files not in SUMMARY.md
- **MDBOOK007-MDBOOK012** - Validate include directives and syntax

## Auto-Fix Rules

**41 rules** support automatic fixing with `--fix`:

### Whitespace & Formatting

- **MD009** - Remove trailing spaces
- **MD010** - Convert tabs to spaces  
- **MD012** - Remove excess blank lines
- **MD047** - Add final newline

### Headings

- **MD018** - Add space after `#`
- **MD019** - Fix multiple spaces after `#`
- **MD020** - Remove spaces in `###Heading###`
- **MD021** - Fix spaces in `### Heading ###`
- **MD022** - Standardize heading spacing (lines before/after)
- **MD023** - Remove heading indentation

### Lists & Blockquotes

- **MD027** - Fix blockquote spacing
- **MD028** - Add blockquote markers to blank lines inside blockquotes
- **MD029** - Fix ordered list prefix consistency (sequential vs all-ones)
- **MD030** - Fix list marker spacing

### Links & Images

- **MD034** - Convert bare URLs to links
- **MD045** - Add placeholder alt text to images

### Code & Formatting

- **MD014** - Remove dollar sign prompts from shell commands
- **MD035** - Standardize horizontal rule style (---, ***, ___)
- **MD048** - Standardize code fence style (backticks vs tildes)
- **MD050** - Standardize strong emphasis style (** vs __)

### Tables

- **MD055** - Fix table pipe style (leading/trailing pipes)
- **MD056** - Balance table columns by adding empty cells

## Rule Configuration

Many rules support customization through configuration files. Common patterns:

### Line Length (MD013)

```toml
[MD013]
line_length = 120      # Default: 80
length_mode = "visual" # "strict" (default) or "visual" (excludes URLs from count)
code_blocks = false    # Ignore code blocks
tables = false         # Ignore tables  
headings = false       # Ignore headings
```

### List Indentation (MD007)

```toml
[MD007]
indent = 4             # Default: 2 spaces per level
start_indented = true  # Allow first level to be indented
```

### Trailing Spaces (MD009)

```toml
[MD009]
br_spaces = 2          # Allow 2 spaces for line breaks
strict = false         # Allow configured line breaks
```

### Heading Style (MD003)

```toml
[MD003]
style = "atx"          # Options: "atx", "setext", "atx_closed"
```

See [Configuration Reference](./configuration-reference.md) for complete options.

## Disabling Rules

### Global Disable

```toml
# .mdbook-lint.toml
disabled_rules = ["MD013", "MD033"]
```

### Inline Disable Comments

```markdown
<!-- mdbook-lint-disable MD013 -->
This line can be very long without triggering violations.
<!-- mdbook-lint-enable MD013 -->

<!-- mdbook-lint-disable-next-line MD034 -->
https://example.com can be a bare URL here.
```

### File-level Disable

```markdown
<!-- mdbook-lint-disable-file MD001 MD022 -->
# This file ignores heading increment and blank line rules

### Can skip levels when file-level disabled
```

## Using Auto-Fix

Auto-fix rules can be applied with CLI commands:

```bash
# Fix all auto-fixable violations
mdbook-lint lint --fix docs/

# Preview what would be fixed
mdbook-lint lint --fix --dry-run docs/

# Apply potentially unsafe fixes too
mdbook-lint lint --fix-unsafe docs/

# Fix without creating backups
mdbook-lint lint --fix --no-backup docs/
```

## Rule Categories

Rules are organized into logical categories:

- **Structure** (7 rules) - Document organization and hierarchy
- **Style** (8 rules) - Formatting and visual consistency  
- **Whitespace** (6 rules) - Spacing, blank lines, and indentation
- **Headings** (11 rules) - Heading format and structure
- **Lists** (8 rules) - List formatting and indentation
- **Links** (9 rules) - Link syntax and validation
- **Code** (8 rules) - Code blocks and inline code
- **Emphasis** (4 rules) - Bold and italic formatting
- **Tables** (3 rules) - Table structure and formatting
- **Images** (1 rule) - Image alt text and formatting
- **HTML** (1 rule) - Inline HTML usage
- **Blockquotes** (2 rules) - Blockquote formatting

## Getting Help

### List All Rules

```bash
# Basic list
mdbook-lint rules

# Detailed descriptions
mdbook-lint rules --detailed

# Only enabled rules for your config
mdbook-lint rules --enabled

# JSON format for scripting
mdbook-lint rules --format json
```

### Rule-Specific Help

```bash
# Get help for specific rule
mdbook-lint help MD009

# Check rule configuration
mdbook-lint explain MD013
```

## Rule Development

Interested in contributing new rules? See our guides:

- [Contributing Guide](./contributing.html) - How to contribute
- [Rule Development](./contributing.html#rule-development) - Creating new rules
- [API Documentation](./api-documentation.html) - Core interfaces

## Next Steps

- **Get Started**: Try [auto-fix](./cli-usage.html#auto-fix) on your documents
- **Customize**: Set up your [configuration](./configuration-reference.html)

- **Integrate**: Add to your [mdBook project](./mdbook-integration.html)
- **Learn More**: Read individual rule pages for detailed examples
