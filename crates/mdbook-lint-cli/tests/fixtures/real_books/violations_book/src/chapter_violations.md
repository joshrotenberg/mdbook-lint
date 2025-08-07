# Chapter with Violations

This chapter intentionally contains various linting violations for testing purposes.

### Skipped heading level (MD001 violation)

This is a level 3 heading directly after a level 1 heading, which violates MD001.

##### And this skips even more levels (MD001)

Multiple heading level violations in one document.

## Inconsistent List Formatting

*   First item with asterisk (MD004)
-   Second item with dash (MD004)
+   Third item with plus (MD004)

## Line Length Issues

This line is intentionally way too long and should trigger MD013 line length rule when configured with default settings because it exceeds the typical 80 character limit that is commonly used in markdown linting tools.

## Code Block Issues

```
function withoutLanguageTag() {
    console.log("This should trigger MDBOOK001 - missing language tag");
    return "violation";
}
```

Another code block without language specification:

```
#!/bin/bash
echo "This shell script also lacks a language tag"
echo "Should trigger MDBOOK001 as well"
```

## Link Issues

[Broken internal link](./does-not-exist.md) - should trigger MDBOOK002

[Another broken link](../missing/file.md) - also MDBOOK002

## Spacing Issues

>Block quote without proper spacing before
Should trigger MD022.

## Trailing Whitespace

This line has trailing spaces.   
This line also has trailing spaces.	

## Mixed Code Block Styles

Here's a code block with language:

```rust
fn proper_code() {
    println!("This one is fine");
}
```

But then we have another without:

```
def python_without_lang():
    print("This triggers MDBOOK001")
```

## Additional Violations

- Unordered list item
1. Followed immediately by ordered list (MD004)
2. Which creates inconsistency

## Blank Line Issues

Text followed immediately by heading without blank line
## This Heading Violates MD022

## Final Section

This chapter should trigger multiple violations:
- MD001: Multiple heading level skips
- MD004: Inconsistent list markers  
- MD013: Lines too long
- MD022: Headings without surrounding blank lines
- MDBOOK001: Multiple code blocks without language tags
- MDBOOK002: Multiple broken links

Some final content to end the chapter.