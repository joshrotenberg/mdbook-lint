//! Violation types for mdbook-lint
//!
//! This module contains the core types for representing linting violations.

/// A suggested fix for a violation
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Fix {
    /// Description of what the fix does
    pub description: String,
    /// The replacement text (None means delete)
    pub replacement: Option<String>,
    /// Start position of the text to replace
    pub start: Position,
    /// End position of the text to replace  
    pub end: Position,
}

/// Position in a document
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct Position {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
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
}
