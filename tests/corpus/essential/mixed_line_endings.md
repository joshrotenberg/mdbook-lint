# Mixed Line Endings Test

This document tests handling of different line ending styles.

## Windows CRLF
Content with Windows line endings.

## Unix LF
Content with Unix line endings.

## Mixed Content
Some lines have different endings.
This line might have CRLF.
This line might have LF only.

The linter should handle all variants gracefully.