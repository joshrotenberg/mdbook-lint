# Tool Comparison: mdbook-lint vs markdownlint

This document analyzes the differences in linting behavior between mdbook-lint and markdownlint when run on the same documentation.

## Performance Metrics

| Metric | mdbook-lint | markdownlint | Difference |
|--------|-------------|--------------|------------|
| **Execution Time** | 0.101s | 0.283s | mdbook-lint is **2.8x faster** |
| **Total Violations** | 1,390 | 818 | mdbook-lint finds **70% more** |
| **Standard Rules Only** | 1,018 | 818 | mdbook-lint finds **24% more** |

## Rule-by-Rule Analysis

### Rules with Similar Detection (±10% difference)

These rules show consistent behavior between both tools:

| Rule | Description | mdbook-lint | markdownlint |
|------|-------------|-------------|--------------|
| MD022 | Headings surrounded by blanks | 177 | 177 |
| MD031 | Code blocks surrounded by blanks | 200 | 202 |
| MD032 | Lists surrounded by blanks | 181 | 174 |
| MD047 | Files end with newline | 37 | 37 |
| MD051 | Link fragments | 47 | 47 |

### Major Discrepancies

#### Rules Where mdbook-lint Finds More Violations

| Rule | Description | mdbook-lint | markdownlint | Analysis |
|------|-------------|-------------|--------------|----------|
| **MD013** | Line length | 179 | 115 | mdbook-lint is **55% stricter** |
| **MD007** | List indentation | 46 | 0 | markdownlint doesn't check inside code blocks |
| **MD052** | Reference links | 24 | 0 | mdbook-lint validates undefined references |
| **MD006** | Lists start at beginning | 21 | 0 | mdbook-lint enforces list positioning |
| **MD058** | Tables surrounded by blanks | 11 | 2 | mdbook-lint has stricter table detection |

#### Rules Only Detected by mdbook-lint

These standard markdown rules are caught by mdbook-lint but not markdownlint in our docs:

- **MD014** (1): Dollar signs in shell code
- **MD018** (8): No space after hash in headings
- **MD019** (10): Multiple spaces after hash
- **MD020** (14): No space in closed headings
- **MD021** (8): Multiple spaces in closed headings
- **MD023** (12): Headings not at line beginning
- **MD027** (4): Multiple spaces after blockquote
- **MD028** (1): Blank line inside blockquote
- **MD030** (1): Spaces after list markers
- **MD033** (6): Inline HTML
- **MD035** (2): Horizontal rule style
- **MD039** (1): Spaces inside link text
- **MD044** (10): Proper names capitalization
- **MD050** (1): Strong style consistency

## Root Cause Analysis

### 1. Code Block Processing (MD007)

**Issue**: mdbook-lint reports 46 MD007 violations, markdownlint reports 0

**Example**:

```yaml
steps:
  - uses: actions/checkout@v4  # mdbook-lint flags this indentation
```

**Analysis**: mdbook-lint appears to be checking list indentation rules *inside* code blocks, which is incorrect. Code blocks should be treated as literal content.

### 2. Line Length Strictness (MD013)

**Issue**: mdbook-lint finds 179 violations vs markdownlint's 115

**Analysis**: Both tools use 80-character default, but mdbook-lint may:

- Count differently (e.g., including/excluding certain characters)
- Check more contexts (e.g., inside certain structures)
- Have different handling of Unicode or special characters

### 3. Reference Link Validation (MD052)

**Issue**: mdbook-lint finds 24 violations, markdownlint finds 0

**Example from contributing.md:335-337**:

```markdown
[ ]  # mdbook-lint flags as undefined reference
```

**Analysis**: mdbook-lint validates that reference links actually have definitions, while markdownlint may only check syntax.

### 4. Whitespace Rules (MD018-MD021, MD023, MD027)

**Issue**: mdbook-lint finds 42 total violations across these rules, markdownlint finds 0

**Analysis**: mdbook-lint has more comprehensive whitespace checking around:

- Heading markers (MD018-MD021)
- Heading indentation (MD023)
- Blockquote markers (MD027)

## mdBook-Specific Rules

mdbook-lint includes 372 additional violations from mdBook-specific rules:

| Rule | Count | Purpose |
|------|-------|---------|
| MDBOOK002 | 157 | Internal link validation |
| MDBOOK007 | 75 | File include syntax |
| MDBOOK005 | 46 | Orphaned files detection |
| MDBOOK001 | 30 | Code blocks need language tags |
| MDBOOK012 | 21 | File include ranges |
| MDBOOK008 | 13 | Rustdoc include validation |
| Others | 30 | Various mdBook features |

## Recommendations

### For mdbook-lint

1. **Fix MD007**: Should not check list indentation inside code blocks
2. **Document MD013**: Clarify how line length is calculated
3. **Configuration alignment**: Consider a `markdownlint-compatible` mode that matches behavior exactly

### For Users

1. **Choose based on needs**:


- **mdbook-lint**: Better for mdBook projects, stricter checking, faster performance

- **markdownlint**: Better for general markdown, more mature, wider ecosystem

2. **Configuration tips**:


- Disable MD007 in mdbook-lint if false positives in code blocks are problematic

- Adjust MD013 line length if 80 characters is too strict

- Use `markdownlint-compatible` flag for closer behavior matching

## Conclusion

mdbook-lint is significantly stricter and faster than markdownlint, finding 70% more violations overall. The main differences stem from:

1. **Bug**: MD007 checking inside code blocks (should be fixed)
2. **Design**: Stricter validation of references, whitespace, and formatting
3. **Feature**: mdBook-specific rules add valuable checks for mdBook projects

For mdBook projects, mdbook-lint provides superior coverage. For general markdown, the choice depends on whether stricter checking is desired.
