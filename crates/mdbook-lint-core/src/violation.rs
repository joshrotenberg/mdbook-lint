//! Violation types for mdbook-lint
//!
//! This module contains the core types for representing linting violations.

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
        };

        assert_eq!(violation.rule_id, "MD001");
        assert_eq!(violation.rule_name, "heading-increment");
        assert_eq!(violation.line, 5);
        assert_eq!(violation.column, 1);
        assert_eq!(violation.severity, Severity::Warning);
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
        };

        let violation2 = Violation {
            rule_id: "MD001".to_string(),
            rule_name: "heading-increment".to_string(),
            message: "Test message".to_string(),
            line: 1,
            column: 1,
            severity: Severity::Warning,
        };

        let violation3 = Violation {
            rule_id: "MD002".to_string(),
            rule_name: "first-heading-h1".to_string(),
            message: "Different message".to_string(),
            line: 2,
            column: 1,
            severity: Severity::Error,
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
            };

            // Test that display format includes severity
            let display_str = format!("{violation}");
            assert!(display_str.contains(&format!("{severity}")));
        }
    }
}
