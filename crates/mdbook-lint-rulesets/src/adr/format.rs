//! ADR format detection utilities
//!
//! Provides functionality to detect whether an ADR follows the Nygard format
//! or the MADR 4.0 format.

use regex::Regex;
use std::sync::LazyLock;

/// The format of an Architecture Decision Record
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AdrFormat {
    /// Nygard format: Plain markdown with "Date:" line and sections like Status, Context, Decision
    Nygard,
    /// MADR 4.0 format: YAML frontmatter with status/date fields
    Madr4,
    /// Auto-detect format based on content
    #[default]
    Auto,
}

impl std::fmt::Display for AdrFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdrFormat::Nygard => write!(f, "nygard"),
            AdrFormat::Madr4 => write!(f, "madr"),
            AdrFormat::Auto => write!(f, "auto"),
        }
    }
}

impl std::str::FromStr for AdrFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "nygard" => Ok(AdrFormat::Nygard),
            "madr" | "madr4" => Ok(AdrFormat::Madr4),
            "auto" => Ok(AdrFormat::Auto),
            _ => Err(format!("Unknown ADR format: {}", s)),
        }
    }
}

/// Regex for detecting Nygard-style title: "# N. Title" or "# N - Title"
static NYGARD_TITLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^#\s+(\d+)[.\-\s]+\s*(.+)$").expect("Invalid regex"));

/// Detect the ADR format based on content
///
/// Returns `AdrFormat::Madr4` if YAML frontmatter is present (starts with `---`),
/// otherwise returns `AdrFormat::Nygard`.
pub fn detect_format(content: &str) -> AdrFormat {
    let trimmed = content.trim_start();

    // MADR 4.0 uses YAML frontmatter
    if trimmed.starts_with("---") {
        return AdrFormat::Madr4;
    }

    // Default to Nygard format for plain markdown
    AdrFormat::Nygard
}

/// Check if a document looks like an ADR based on content or path
///
/// Returns true if the document appears to be an Architecture Decision Record:
/// - Has YAML frontmatter with a `status` field (MADR)
/// - Has a numbered title like "# 1. Title" (Nygard)
/// - Has a path containing "adr" or "adrs" directory
pub fn is_adr_document(content: &str, file_path: Option<&std::path::Path>) -> bool {
    // Check if in an ADR directory (handle both absolute and relative paths)
    if let Some(path) = file_path {
        let path_str = path.to_string_lossy().to_lowercase();
        // Check for ADR directory anywhere in path, including at start for relative paths
        if path_str.contains("/adr/")
            || path_str.contains("/adrs/")
            || path_str.contains("\\adr\\")
            || path_str.contains("\\adrs\\")
            || path_str.starts_with("adr/")
            || path_str.starts_with("adrs/")
            || path_str.starts_with("adr\\")
            || path_str.starts_with("adrs\\")
        {
            return true;
        }
    }

    // Check for MADR frontmatter with status field
    let trimmed = content.trim_start();
    if let Some(after_open) = trimmed.strip_prefix("---") {
        // Simple check for status in frontmatter
        if let Some(end) = after_open.find("---") {
            let frontmatter = &after_open[..end];
            if frontmatter.lines().any(|line| {
                let line = line.trim();
                line.starts_with("status:") || line.starts_with("status :")
            }) {
                return true;
            }
        }
    }

    // Check for Nygard-style numbered title
    for line in content.lines().take(5) {
        if is_nygard_title(line) {
            return true;
        }
    }

    false
}

/// Extract the ADR number from a Nygard-style title
///
/// Nygard titles follow the pattern "# N. Title" or "# N - Title"
pub fn extract_nygard_number(title_line: &str) -> Option<u32> {
    NYGARD_TITLE_REGEX
        .captures(title_line)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

/// Extract the title text from a Nygard-style title line
///
/// Returns the title without the number prefix
pub fn extract_nygard_title(title_line: &str) -> Option<&str> {
    NYGARD_TITLE_REGEX
        .captures(title_line)
        .and_then(|caps| caps.get(2))
        .map(|m| m.as_str().trim())
}

/// Check if a line matches the Nygard title format
pub fn is_nygard_title(line: &str) -> bool {
    NYGARD_TITLE_REGEX.is_match(line)
}

/// Parsed information from an ADR document
#[derive(Debug, Clone)]
pub struct ParsedAdr {
    /// Detected format
    pub format: AdrFormat,
    /// ADR number (if extractable from title or filename)
    pub number: Option<u32>,
    /// Title text (without number prefix for Nygard)
    pub title: Option<String>,
    /// Status value
    pub status: Option<String>,
    /// Date value
    pub date: Option<String>,
    /// Line number where the H1 title is found (1-indexed)
    pub title_line: Option<usize>,
    /// Line number where status section/field is found (1-indexed)
    pub status_line: Option<usize>,
    /// Line number where date is found (1-indexed)
    pub date_line: Option<usize>,
}

impl ParsedAdr {
    /// Create a new empty ParsedAdr
    pub fn new(format: AdrFormat) -> Self {
        Self {
            format,
            number: None,
            title: None,
            status: None,
            date: None,
            title_line: None,
            status_line: None,
            date_line: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format_madr() {
        let content = r#"---
status: accepted
date: 2024-01-15
---

# Use PostgreSQL
"#;
        assert_eq!(detect_format(content), AdrFormat::Madr4);
    }

    #[test]
    fn test_detect_format_nygard() {
        let content = r#"# 1. Use Rust for implementation

Date: 2024-01-15

## Status

Accepted
"#;
        assert_eq!(detect_format(content), AdrFormat::Nygard);
    }

    #[test]
    fn test_detect_format_with_leading_whitespace() {
        // Leading whitespace should be ignored
        let content = "   \n\n---\nstatus: accepted\n---\n";
        assert_eq!(detect_format(content), AdrFormat::Madr4);
    }

    #[test]
    fn test_extract_nygard_number() {
        assert_eq!(extract_nygard_number("# 1. Use Rust"), Some(1));
        assert_eq!(extract_nygard_number("# 42. Some Decision"), Some(42));
        assert_eq!(extract_nygard_number("# 1 - Use Rust"), Some(1));
        assert_eq!(extract_nygard_number("# Use Rust"), None);
        assert_eq!(extract_nygard_number("## 1. Section"), None);
    }

    #[test]
    fn test_extract_nygard_title() {
        assert_eq!(extract_nygard_title("# 1. Use Rust"), Some("Use Rust"));
        assert_eq!(
            extract_nygard_title("# 42. Some Decision"),
            Some("Some Decision")
        );
        assert_eq!(extract_nygard_title("# 1 - Use Rust"), Some("Use Rust"));
        assert_eq!(extract_nygard_title("# Use Rust"), None);
    }

    #[test]
    fn test_is_nygard_title() {
        assert!(is_nygard_title("# 1. Use Rust"));
        assert!(is_nygard_title("# 42. Some Decision"));
        assert!(is_nygard_title("# 1 - Use Rust"));
        assert!(!is_nygard_title("# Use Rust"));
        assert!(!is_nygard_title("## 1. Section"));
    }

    #[test]
    fn test_format_from_str() {
        assert_eq!("nygard".parse::<AdrFormat>().unwrap(), AdrFormat::Nygard);
        assert_eq!("madr".parse::<AdrFormat>().unwrap(), AdrFormat::Madr4);
        assert_eq!("madr4".parse::<AdrFormat>().unwrap(), AdrFormat::Madr4);
        assert_eq!("auto".parse::<AdrFormat>().unwrap(), AdrFormat::Auto);
        assert_eq!("NYGARD".parse::<AdrFormat>().unwrap(), AdrFormat::Nygard);
        assert!("unknown".parse::<AdrFormat>().is_err());
    }

    #[test]
    fn test_format_display() {
        assert_eq!(format!("{}", AdrFormat::Nygard), "nygard");
        assert_eq!(format!("{}", AdrFormat::Madr4), "madr");
        assert_eq!(format!("{}", AdrFormat::Auto), "auto");
    }
}
