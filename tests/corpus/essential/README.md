# Essential Test Corpus

This directory contains a minimal set of essential test files for corpus testing.

## Files

- `empty_file.md` - Completely empty file
- `unicode_content.md` - Various Unicode characters and international text  
- `large_file.md` - Reasonably large document for performance testing
- `mixed_line_endings.md` - Different line ending styles
- `known_violations.md` - File with intentional rule violations for testing

## Purpose

These files provide focused correctness testing without the complexity of the previous extensive corpus framework. They test:

1. **Edge cases**: Empty files, Unicode, line endings
2. **Performance**: Large documents that should still be fast
3. **Correctness**: Known violations to ensure rules work
4. **Stability**: Various inputs that should never crash the linter

This minimal corpus is designed to be:
- Fast to run locally
- No external dependencies  
- Focused on essential test cases
- Easy to understand and maintain