//! Violation types for mdbook-lint
//!
//! This module contains the core types for representing linting violations.

use std::ops::Range;

/// A suggested fix for a violation.
///
/// `start` and `end` form an exact, half-open range: the character at `start`
/// is replaced and the character at `end` is not. Equal positions represent
/// an insertion. Line terminators are never included implicitly; a range that
/// consumes a line terminator ends at column 1 of the following line.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Fix {
    /// Description of what the fix does
    pub description: String,
    /// The replacement text (None means delete)
    pub replacement: Option<String>,
    /// Inclusive start position of the text to replace
    pub start: Position,
    /// Exclusive end position of the text to replace
    pub end: Position,
}

/// A position in a document.
///
/// Lines and columns are 1-based. Columns count Unicode scalar values in the
/// line's content; they are not UTF-8 byte offsets, grapheme clusters, or
/// display cells. A line containing `"café"` therefore ends at column 5.
///
/// The end-of-line column is immediately before the complete line terminator.
/// Column 1 of the following line is immediately after it. Consequently CRLF
/// is atomic: no valid `Position` can point between `\r` and `\n`.
///
/// EOF is represented by the final line's end column when the document has no
/// trailing terminator, or by column 1 of the next line when it does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct Position {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
}

impl Position {
    /// Return column 1 of `line`.
    pub const fn line_start(line: usize) -> Self {
        Self { line, column: 1 }
    }

    /// Return the position immediately after `line_content` and before its
    /// terminator, if any.
    pub fn line_end(line: usize, line_content: &str) -> Self {
        Self {
            line,
            column: line_content.chars().count() + 1,
        }
    }

    /// Convert a zero-based UTF-8 byte offset within one line to a scalar-based
    /// document position.
    ///
    /// Returns `None` if `byte_offset` is outside `line_content` or is not a
    /// UTF-8 character boundary.
    pub fn from_byte_offset_in_line(
        line: usize,
        line_content: &str,
        byte_offset: usize,
    ) -> Option<Self> {
        if line == 0
            || byte_offset > line_content.len()
            || !line_content.is_char_boundary(byte_offset)
        {
            return None;
        }

        Some(Self {
            line,
            column: line_content[..byte_offset].chars().count() + 1,
        })
    }

    /// Resolve this position to a zero-based UTF-8 byte offset in `content`.
    ///
    /// Invalid line or column numbers, including positions inside CRLF, return
    /// `None`.
    pub fn to_byte_offset(self, content: &str) -> Option<usize> {
        if self.line == 0 || self.column == 0 {
            return None;
        }

        let (line_start, line_end) = logical_line_bounds(content, self.line)?;
        let target = self.column - 1;
        let line_content = &content[line_start..line_end];

        for (scalar_index, (byte_offset, _)) in line_content.char_indices().enumerate() {
            if scalar_index == target {
                return Some(line_start + byte_offset);
            }
        }

        (line_content.chars().count() == target).then_some(line_end)
    }

    /// Convert a zero-based UTF-8 byte offset in `content` to a document
    /// position.
    ///
    /// Offsets outside the document, in the middle of a UTF-8 scalar, or
    /// between the two bytes of a CRLF terminator return `None`.
    pub fn from_byte_offset(content: &str, byte_offset: usize) -> Option<Self> {
        if byte_offset > content.len() || !content.is_char_boundary(byte_offset) {
            return None;
        }

        let mut line = 1;
        let mut line_start = 0;
        loop {
            let newline = content[line_start..]
                .find('\n')
                .map(|relative| line_start + relative);
            let line_end = match newline {
                Some(newline_offset)
                    if content.as_bytes().get(newline_offset.wrapping_sub(1)) == Some(&b'\r') =>
                {
                    newline_offset - 1
                }
                Some(newline_offset) => newline_offset,
                None => content.len(),
            };

            if byte_offset >= line_start && byte_offset <= line_end {
                return Some(Self {
                    line,
                    column: content[line_start..byte_offset].chars().count() + 1,
                });
            }

            let next_line_start = newline? + 1;
            if byte_offset < next_line_start {
                // The only representable gap is between '\r' and '\n'.
                return None;
            }
            line += 1;
            line_start = next_line_start;
        }
    }
}

impl Fix {
    /// Construct an exact insertion at `position`.
    pub fn insertion(
        description: impl Into<String>,
        replacement: impl Into<String>,
        position: Position,
    ) -> Self {
        Self {
            description: description.into(),
            replacement: Some(replacement.into()),
            start: position,
            end: position,
        }
    }

    /// Construct a replacement for one complete source line.
    ///
    /// `replacement` contains line content only. When `line_ending` is present,
    /// the helper appends that exact terminator and makes the range end at
    /// column 1 of the following line. With `None`, the range ends before EOF
    /// and no terminator is added.
    pub fn line_replacement(
        description: impl Into<String>,
        replacement: impl Into<String>,
        line: usize,
        original_line: &str,
        line_ending: Option<&str>,
    ) -> Self {
        let mut replacement = replacement.into();
        let end = if let Some(line_ending) = line_ending {
            replacement.push_str(line_ending);
            Position::line_start(line + 1)
        } else {
            Position::line_end(line, original_line)
        };

        Self {
            description: description.into(),
            replacement: Some(replacement),
            start: Position::line_start(line),
            end,
        }
    }

    /// Construct a replacement spanning complete source lines.
    ///
    /// This is the multi-line counterpart to [`Self::line_replacement`]. The
    /// replacement contains its internal line terminators but not the terminator
    /// of `end_line`; that final terminator is supplied separately so its
    /// inclusion is explicit and CRLF can be preserved.
    pub fn line_range_replacement(
        description: impl Into<String>,
        replacement: impl Into<String>,
        start_line: usize,
        end_line: usize,
        original_end_line: &str,
        end_line_ending: Option<&str>,
    ) -> Self {
        let mut replacement = replacement.into();
        let end = if let Some(line_ending) = end_line_ending {
            replacement.push_str(line_ending);
            Position::line_start(end_line + 1)
        } else {
            Position::line_end(end_line, original_end_line)
        };

        Self {
            description: description.into(),
            replacement: Some(replacement),
            start: Position::line_start(start_line),
            end,
        }
    }

    /// Resolve the exact half-open fix range to UTF-8 byte offsets.
    ///
    /// This is the canonical conversion for library embedders. The returned
    /// range is validated for ordering, document bounds, UTF-8 boundaries, and
    /// CRLF atomicity.
    pub fn byte_range(&self, content: &str) -> Option<Range<usize>> {
        let start = self.start.to_byte_offset(content)?;
        let end = self.end.to_byte_offset(content)?;
        (start <= end).then_some(start..end)
    }
}

fn logical_line_bounds(content: &str, target_line: usize) -> Option<(usize, usize)> {
    if target_line == 0 {
        return None;
    }

    let mut line = 1;
    let mut line_start = 0;
    loop {
        let newline = content[line_start..]
            .find('\n')
            .map(|relative| line_start + relative);
        let line_end = match newline {
            Some(newline_offset)
                if content.as_bytes().get(newline_offset.wrapping_sub(1)) == Some(&b'\r') =>
            {
                newline_offset - 1
            }
            Some(newline_offset) => newline_offset,
            None => content.len(),
        };

        if line == target_line {
            return Some((line_start, line_end));
        }

        line_start = newline? + 1;
        line += 1;
    }
}

/// A violation found during linting
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Violation {
    /// Rule identifier (e.g., "MD001")
    pub rule_id: String,
    /// Human-readable rule name (e.g., "heading-increment")
    pub rule_name: String,
    /// Description of the violation
    pub message: String,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Severity level
    pub severity: Severity,
    /// Optional fix for this violation
    pub fix: Option<Fix>,
}

/// Severity levels for violations
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum Severity {
    /// Informational message
    Info,
    /// Warning that should be addressed
    Warning,
    /// Error that must be fixed
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}: {}/{}: {}",
            self.line, self.column, self.severity, self.rule_id, self.rule_name, self.message
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::Info), "info");
        assert_eq!(format!("{}", Severity::Warning), "warning");
        assert_eq!(format!("{}", Severity::Error), "error");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Info < Severity::Error);
    }

    #[test]
    fn test_violation_creation() {
        let violation = Violation {
            rule_id: "MD001".to_string(),
            rule_name: "heading-increment".to_string(),
            message: "Heading levels should only increment by one level at a time".to_string(),
            line: 5,
            column: 1,
            severity: Severity::Warning,
            fix: None,
        };

        assert_eq!(violation.rule_id, "MD001");
        assert_eq!(violation.rule_name, "heading-increment");
        assert_eq!(violation.line, 5);
        assert_eq!(violation.column, 1);
        assert_eq!(violation.severity, Severity::Warning);
        assert_eq!(violation.fix, None);
    }

    #[test]
    fn test_violation_display() {
        let violation = Violation {
            rule_id: "MD013".to_string(),
            rule_name: "line-length".to_string(),
            message: "Line too long".to_string(),
            line: 10,
            column: 81,
            severity: Severity::Error,
            fix: None,
        };

        let expected = "10:81:error: MD013/line-length: Line too long";
        assert_eq!(format!("{violation}"), expected);
    }

    #[test]
    fn test_violation_equality() {
        let violation1 = Violation {
            rule_id: "MD001".to_string(),
            rule_name: "heading-increment".to_string(),
            message: "Test message".to_string(),
            line: 1,
            column: 1,
            severity: Severity::Warning,
            fix: None,
        };

        let violation2 = Violation {
            rule_id: "MD001".to_string(),
            rule_name: "heading-increment".to_string(),
            message: "Test message".to_string(),
            line: 1,
            column: 1,
            severity: Severity::Warning,
            fix: None,
        };

        let violation3 = Violation {
            rule_id: "MD002".to_string(),
            rule_name: "first-heading-h1".to_string(),
            message: "Different message".to_string(),
            line: 2,
            column: 1,
            severity: Severity::Error,
            fix: None,
        };

        assert_eq!(violation1, violation2);
        assert_ne!(violation1, violation3);
    }

    #[test]
    fn test_violation_clone() {
        let original = Violation {
            rule_id: "MD040".to_string(),
            rule_name: "fenced-code-language".to_string(),
            message: "Fenced code blocks should have a language specified".to_string(),
            line: 15,
            column: 3,
            severity: Severity::Info,
            fix: None,
        };

        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_violation_debug() {
        let violation = Violation {
            rule_id: "MD025".to_string(),
            rule_name: "single-h1".to_string(),
            message: "Multiple top level headings in the same document".to_string(),
            line: 20,
            column: 1,
            severity: Severity::Warning,
            fix: None,
        };

        let debug_str = format!("{violation:?}");
        assert!(debug_str.contains("MD025"));
        assert!(debug_str.contains("single-h1"));
        assert!(debug_str.contains("Multiple top level headings"));
        assert!(debug_str.contains("line: 20"));
        assert!(debug_str.contains("column: 1"));
        assert!(debug_str.contains("Warning"));
    }

    #[test]
    fn test_all_severity_variants() {
        let severities = [Severity::Info, Severity::Warning, Severity::Error];

        for severity in &severities {
            let violation = Violation {
                rule_id: "TEST".to_string(),
                rule_name: "test-rule".to_string(),
                message: "Test message".to_string(),
                line: 1,
                column: 1,
                severity: *severity,
                fix: None,
            };

            // Test that display format includes severity
            let display_str = format!("{violation}");
            assert!(display_str.contains(&format!("{severity}")));
        }
    }

    #[test]
    fn test_violation_with_fix() {
        let fix = Fix {
            description: "Replace tab with spaces".to_string(),
            replacement: Some("    ".to_string()),
            start: Position {
                line: 5,
                column: 10,
            },
            end: Position {
                line: 5,
                column: 11,
            },
        };

        let violation = Violation {
            rule_id: "MD010".to_string(),
            rule_name: "no-hard-tabs".to_string(),
            message: "Hard tab found".to_string(),
            line: 5,
            column: 10,
            severity: Severity::Warning,
            fix: Some(fix.clone()),
        };

        assert_eq!(violation.fix, Some(fix));
        assert!(violation.fix.is_some());

        let fix_ref = violation.fix.as_ref().unwrap();
        assert_eq!(fix_ref.description, "Replace tab with spaces");
        assert_eq!(fix_ref.replacement, Some("    ".to_string()));
        assert_eq!(fix_ref.start.line, 5);
        assert_eq!(fix_ref.start.column, 10);
        assert_eq!(fix_ref.end.line, 5);
        assert_eq!(fix_ref.end.column, 11);
    }

    #[test]
    fn test_fix_delete_operation() {
        let fix = Fix {
            description: "Remove extra newlines".to_string(),
            replacement: None, // None means delete
            start: Position {
                line: 10,
                column: 1,
            },
            end: Position {
                line: 12,
                column: 1,
            },
        };

        assert_eq!(fix.replacement, None);
        assert_eq!(fix.description, "Remove extra newlines");
    }

    #[test]
    fn position_columns_count_unicode_scalars() {
        let content = "é🙂x\nβ";

        assert_eq!(
            Position { line: 1, column: 1 }.to_byte_offset(content),
            Some(0)
        );
        assert_eq!(
            Position { line: 1, column: 2 }.to_byte_offset(content),
            Some(2)
        );
        assert_eq!(
            Position { line: 1, column: 3 }.to_byte_offset(content),
            Some(6)
        );
        assert_eq!(
            Position { line: 1, column: 4 }.to_byte_offset(content),
            Some(7)
        );
        assert_eq!(
            Position { line: 2, column: 1 }.to_byte_offset(content),
            Some(8)
        );
        assert_eq!(
            Position { line: 2, column: 2 }.to_byte_offset(content),
            Some(10)
        );

        assert_eq!(
            Position::from_byte_offset(content, 6),
            Some(Position { line: 1, column: 3 })
        );
        assert_eq!(
            Position::from_byte_offset(content, 8),
            Some(Position { line: 2, column: 1 })
        );
        assert_eq!(Position::from_byte_offset(content, 1), None);
    }

    #[test]
    fn positions_keep_crlf_atomic() {
        let content = "a\r\né";

        assert_eq!(Position::line_end(1, "a").to_byte_offset(content), Some(1));
        assert_eq!(Position::line_start(2).to_byte_offset(content), Some(3));
        assert_eq!(
            Position::from_byte_offset(content, 1),
            Some(Position { line: 1, column: 2 })
        );
        assert_eq!(Position::from_byte_offset(content, 2), None);
        assert_eq!(
            Position::from_byte_offset(content, 3),
            Some(Position { line: 2, column: 1 })
        );
    }

    #[test]
    fn fix_byte_ranges_are_exact_and_half_open() {
        let content = "é🙂x\n";
        let fix = Fix {
            description: "replace emoji".to_string(),
            replacement: Some("!".to_string()),
            start: Position { line: 1, column: 2 },
            end: Position { line: 1, column: 3 },
        };

        assert_eq!(fix.byte_range(content), Some(2..6));

        let insertion = Fix::insertion("insert", "!", Position { line: 1, column: 2 });
        assert_eq!(insertion.byte_range(content), Some(2..2));
    }

    #[test]
    fn line_replacement_has_explicit_terminator_intent() {
        let lf = Fix::line_replacement("replace", "new", 1, "old", Some("\n"));
        assert_eq!(lf.replacement.as_deref(), Some("new\n"));
        assert_eq!(lf.start, Position::line_start(1));
        assert_eq!(lf.end, Position::line_start(2));
        assert_eq!(lf.byte_range("old\nnext"), Some(0..4));

        let crlf = Fix::line_replacement("replace", "new", 1, "old", Some("\r\n"));
        assert_eq!(crlf.replacement.as_deref(), Some("new\r\n"));
        assert_eq!(crlf.end, Position::line_start(2));
        assert_eq!(crlf.byte_range("old\r\nnext"), Some(0..5));

        let eof = Fix::line_replacement("replace", "new", 1, "old", None);
        assert_eq!(eof.replacement.as_deref(), Some("new"));
        assert_eq!(eof.end, Position::line_end(1, "old"));
        assert_eq!(eof.byte_range("old"), Some(0..3));
    }

    #[test]
    fn eof_positions_cover_empty_and_terminated_documents() {
        assert_eq!(Position::line_start(1).to_byte_offset(""), Some(0));
        assert_eq!(
            Position::from_byte_offset("", 0),
            Some(Position::line_start(1))
        );
        assert_eq!(Position::line_start(2).to_byte_offset("x\n"), Some(2));
        assert_eq!(
            Position::from_byte_offset("x\n", 2),
            Some(Position::line_start(2))
        );
        assert_eq!(Position::line_start(2).to_byte_offset("x"), None);
    }
}
