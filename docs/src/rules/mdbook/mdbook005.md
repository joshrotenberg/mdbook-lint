# MDBOOK005 - Orphaned Files

**Severity**: Warning  
**Category**: mdBook-specific  
**Auto-fix**: Not available

## Rule Description

This rule detects markdown files in your mdBook source directory that are not referenced in SUMMARY.md. Orphaned files won't be included in the built book and represent unused or forgotten content.

## Why This Rule Exists

Detecting orphaned files is important because:

- Identifies forgotten or lost content
- Helps maintain a clean project structure
- Prevents confusion about what's included in the book
- Finds files that should be deleted or added to SUMMARY.md
- Reduces repository size by identifying unused files

## Examples

### ❌ Problematic Structure

```
src/
├── SUMMARY.md
├── introduction.md      ✓ (in SUMMARY.md)
├── chapter1.md         ✓ (in SUMMARY.md)
├── chapter2.md         ✓ (in SUMMARY.md)
├── old-chapter.md      ✗ (orphaned)
├── todo.md            ✗ (orphaned)
└── notes.md           ✗ (orphaned)
```

### ✅ Clean Structure

```
src/
├── SUMMARY.md
├── introduction.md      ✓ (in SUMMARY.md)
├── chapter1.md         ✓ (in SUMMARY.md)
├── chapter2.md         ✓ (in SUMMARY.md)
└── appendix.md         ✓ (in SUMMARY.md)
```

## What This Rule Checks

The rule scans for:

1. All `.md` files in the source directory
2. Files referenced in SUMMARY.md
3. Reports files not in SUMMARY.md as orphaned

### Special Cases

Files that are **not** considered orphaned:

- `SUMMARY.md` itself
- `README.md` (often used as index)
- Files in directories excluded by configuration
- Files matching ignore patterns

## Configuration

```toml
[MDBOOK005]
ignore_patterns = ["drafts/**", "*.backup.md"]  # Patterns to ignore
check_nested = true                             # Check subdirectories (default: true)
exclude_readme = true                           # Don't report README.md (default: true)
```

## Common Scenarios

### Scenario 1: Renamed File

You renamed a chapter but forgot to update SUMMARY.md:

```bash
# Old file still exists but not in SUMMARY.md
src/old-name.md  → orphaned
src/new-name.md  → in SUMMARY.md
```

**Solution**: Delete the old file or update SUMMARY.md

### Scenario 2: Work in Progress

You're drafting new content not ready for inclusion:

```bash
src/draft-chapter.md  → orphaned (intentionally)
```

**Solution**: Move to a drafts folder or add ignore pattern

### Scenario 3: Included Files

You have files that are included by other files:

```bash
src/snippets/example.md  → orphaned (but included via {{#include}})
```

**Solution**: Add to ignore patterns or move to non-source directory

## Handling Orphaned Files

### Option 1: Add to SUMMARY.md

```markdown
# Summary

- [Existing Chapter](./existing.md)
- [Previously Orphaned](./orphaned.md)  <!-- Add this line -->
```

### Option 2: Delete the File

```bash
rm src/orphaned-file.md
```

### Option 3: Move Outside Source Directory

```bash
mkdir archived
mv src/orphaned.md archived/
```

### Option 4: Add to Ignore Patterns

```toml
[MDBOOK005]
ignore_patterns = ["drafts/**", "work-in-progress.md"]
```

## When to Disable

Consider disabling this rule if:

- You intentionally keep reference files in the source directory
- Your build process dynamically generates SUMMARY.md
- You use many include files that aren't directly referenced
- You're in the middle of major restructuring

### Disable in Config

```toml
# .mdbook-lint.toml
disabled_rules = ["MDBOOK005"]
```

### Disable for Specific Directories

```toml
[[overrides]]
path = "src/reference/**"
disabled_rules = ["MDBOOK005"]
```

## Best Practices

1. **Regular cleanup**: Periodically review and remove orphaned files
2. **Use drafts folder**: Keep work-in-progress in a separate directory
3. **Document intentional orphans**: Add comments explaining why files are kept
4. **Version control**: Check git history before deleting orphaned files

## Related Rules

- [MDBOOK002](./mdbook002.html) - Invalid internal link
- [MDBOOK003](./mdbook003.html) - SUMMARY.md structure
- [MDBOOK006](./mdbook006.html) - Cross-reference validation

## References

- [mdBook - SUMMARY.md](https://rust-lang.github.io/mdBook/format/summary.html)
- [mdBook Directory Structure](https://rust-lang.github.io/mdBook/guide/creating.html)
