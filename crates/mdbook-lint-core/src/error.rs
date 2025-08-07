//! Error types for mdbook-lint
//!
//! This module provides a comprehensive error system that replaces the use of `anyhow::Error`
//! with structured, programmatically-handleable error types. This enables better API design
//! and error recovery for library consumers.
//!
//! # Architecture Overview
//!
//! The error system is built around several key principles:
//!
//! 1. **Structured Errors**: Each error variant contains specific contextual information
//! 2. **Error Categories**: Errors are grouped by domain (Rule, Document, Config, etc.)
//! 3. **Conversion Traits**: Seamless conversion between specialized and general error types
//! 4. **Context Addition**: Rich context can be added at any point in the error chain
//! 5. **Compatibility**: Smooth migration path from `anyhow` with compatibility layers
//!
//! # Error Hierarchy
//!
//! ```text
//! MdBookLintError (main error type)
//! ├── Io(std::io::Error)              - File system operations
//! ├── Parse { line, column, message } - Document parsing errors
//! ├── Config(String)                  - Configuration issues
//! ├── Rule { rule_id, message }       - Rule execution problems
//! ├── Plugin(String)                  - Plugin system errors
//! ├── Document(String)                - Document processing errors
//! ├── Registry(String)                - Rule registry operations
//! ├── Json(serde_json::Error)         - JSON serialization errors
//! ├── Yaml(serde_yaml::Error)         - YAML serialization errors
//! └── Toml(toml::de::Error)           - TOML serialization errors
//! ```
//!
//! # Usage Examples
//!
//! ## Basic Error Creation
//!
//! ```rust
//! use mdbook_lint_core::error::{MdBookLintError, Result};
//!
//! // Create specific errors
//! let parse_err = MdBookLintError::parse_error(10, 5, "Invalid syntax");
//! let rule_err = MdBookLintError::rule_error("MD001", "Heading increment violation");
//! let config_err = MdBookLintError::config_error("Invalid rule configuration");
//! ```
//!
//! ## Error Matching and Handling
//!
//! ```rust
//! use mdbook_lint_core::error::MdBookLintError;
//!
//! fn handle_error(err: MdBookLintError) {
//!     match err {
//!         MdBookLintError::Parse { line, column, message } => {
//!             eprintln!("Parse error at {}:{}: {}", line, column, message);
//!         }
//!         MdBookLintError::Rule { rule_id, message } => {
//!             eprintln!("Rule {} failed: {}", rule_id, message);
//!         }
//!         MdBookLintError::Io(io_err) => {
//!             eprintln!("IO error: {}", io_err);
//!         }
//!         _ => eprintln!("Unknown error: {}", err),
//!     }
//! }
//! ```
//!
//! ## Adding Context
//!
//! ```rust
//! use mdbook_lint_core::error::{ErrorContext, Result};
//!
//! fn process_rule() -> Result<()> {
//!     // ... some operation that might fail
//!     # Ok(())
//! }
//!
//! let result = process_rule().with_rule_context("MD001");
//! ```
//!
//! ## Specialized Error Types
//!
//! For domain-specific operations, use the specialized error types:
//!
//! ```rust
//! use mdbook_lint_core::error::{RuleError, DocumentError, IntoMdBookLintError};
//!
//! // Create specialized errors
//! let rule_err = RuleError::not_found("MD999");
//! let doc_err = DocumentError::too_large(1000000, 500000);
//!
//! // Convert to main error type
//! let result: std::result::Result<(), _> = Err(rule_err);
//! let main_result = result.into_mdbook_lint_error();
//! ```
//!
//! # Migration from anyhow
//!
//! This module provides compatibility layers to ease migration:
//!
//! 1. **From<anyhow::Error>**: Convert anyhow errors to MdBookLintError
//! 2. **Blanket conversion**: MdBookLintError implements std::error::Error, so anyhow can convert it back
//! 3. **Result type alias**: Drop-in replacement for anyhow::Result
//!
//! # Performance Considerations
//!
//! - **Zero-cost when successful**: Error types only allocate when errors occur
//! - **Structured data**: No string parsing needed to extract error information
//! - **Efficient matching**: Enum variants enable efficient error handling
//! - **Context preservation**: Rich context without performance penalty in success cases

use thiserror::Error;

/// Main error type for mdbook-lint operations
#[derive(Debug, Error)]
pub enum MdBookLintError {
    /// IO-related errors (file reading, writing, etc.)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Parsing errors with position information
    #[error("Parse error at line {line}, column {column}: {message}")]
    Parse {
        line: usize,
        column: usize,
        message: String,
    },

    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Rule execution errors
    #[error("Rule error in {rule_id}: {message}")]
    Rule { rule_id: String, message: String },

    /// Plugin system errors
    #[error("Plugin error: {0}")]
    Plugin(String),

    /// Document processing errors
    #[error("Document error: {0}")]
    Document(String),

    /// Registry operation errors
    #[error("Registry error: {0}")]
    Registry(String),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// YAML serialization/deserialization errors
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// TOML serialization/deserialization errors
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    /// Directory traversal errors
    #[error("Directory traversal error: {0}")]
    WalkDir(#[from] walkdir::Error),
}

/// Specialized error type for rule-related operations
#[derive(Debug, Error)]
pub enum RuleError {
    /// Rule not found in registry
    #[error("Rule not found: {rule_id}")]
    NotFound { rule_id: String },

    /// Rule execution failed
    #[error("Rule execution failed: {message}")]
    ExecutionFailed { message: String },

    /// Invalid rule configuration
    #[error("Invalid rule configuration: {message}")]
    InvalidConfig { message: String },

    /// Rule dependency not met
    #[error("Rule dependency not satisfied: {rule_id} requires {dependency}")]
    DependencyNotMet { rule_id: String, dependency: String },

    /// Rule registration conflict
    #[error("Rule registration conflict: rule {rule_id} already exists")]
    RegistrationConflict { rule_id: String },
}

/// Specialized error type for document-related operations
#[derive(Debug, Error)]
pub enum DocumentError {
    /// Failed to read document from file
    #[error("Failed to read document: {path}")]
    ReadFailed { path: String },

    /// Document format is invalid or unsupported
    #[error("Invalid document format")]
    InvalidFormat,

    /// Document exceeds size limits
    #[error("Document too large: {size} bytes (max: {max_size})")]
    TooLarge { size: usize, max_size: usize },

    /// Document parsing failed
    #[error("Failed to parse document: {reason}")]
    ParseFailed { reason: String },

    /// Invalid encoding detected
    #[error("Invalid encoding in document: {path}")]
    InvalidEncoding { path: String },
}

/// Specialized error type for configuration operations
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Configuration file not found
    #[error("Configuration file not found: {path}")]
    NotFound { path: String },

    /// Invalid configuration format
    #[error("Invalid configuration format: {message}")]
    InvalidFormat { message: String },

    /// Configuration validation failed
    #[error("Configuration validation failed: {field} - {message}")]
    ValidationFailed { field: String, message: String },

    /// Unsupported configuration version
    #[error("Unsupported configuration version: {version} (supported: {supported})")]
    UnsupportedVersion { version: String, supported: String },
}

/// Specialized error type for plugin operations
#[derive(Debug, Error)]
pub enum PluginError {
    /// Plugin not found
    #[error("Plugin not found: {plugin_id}")]
    NotFound { plugin_id: String },

    /// Plugin loading failed
    #[error("Failed to load plugin {plugin_id}: {reason}")]
    LoadFailed { plugin_id: String, reason: String },

    /// Plugin initialization failed
    #[error("Plugin initialization failed: {plugin_id}")]
    InitializationFailed { plugin_id: String },

    /// Plugin version incompatibility
    #[error("Plugin version incompatible: {plugin_id} version {version} (required: {required})")]
    VersionIncompatible {
        plugin_id: String,
        version: String,
        required: String,
    },
}

/// Result type alias for mdbook-lint operations
pub type Result<T> = std::result::Result<T, MdBookLintError>;

/// Convenience constructors for common error patterns
impl MdBookLintError {
    /// Create a parse error with position information
    pub fn parse_error(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self::Parse {
            line,
            column,
            message: message.into(),
        }
    }

    /// Create a rule error with context
    pub fn rule_error(rule_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Rule {
            rule_id: rule_id.into(),
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn config_error(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    /// Create a document error
    pub fn document_error(message: impl Into<String>) -> Self {
        Self::Document(message.into())
    }

    /// Create a plugin error
    pub fn plugin_error(message: impl Into<String>) -> Self {
        Self::Plugin(message.into())
    }

    /// Create a registry error
    pub fn registry_error(message: impl Into<String>) -> Self {
        Self::Registry(message.into())
    }
}

/// Convenience constructors for rule errors
impl RuleError {
    /// Create a "not found" error
    pub fn not_found(rule_id: impl Into<String>) -> Self {
        Self::NotFound {
            rule_id: rule_id.into(),
        }
    }

    /// Create an execution failed error
    pub fn execution_failed(message: impl Into<String>) -> Self {
        Self::ExecutionFailed {
            message: message.into(),
        }
    }

    /// Create an invalid config error
    pub fn invalid_config(message: impl Into<String>) -> Self {
        Self::InvalidConfig {
            message: message.into(),
        }
    }

    /// Create a dependency not met error
    pub fn dependency_not_met(rule_id: impl Into<String>, dependency: impl Into<String>) -> Self {
        Self::DependencyNotMet {
            rule_id: rule_id.into(),
            dependency: dependency.into(),
        }
    }

    /// Create a registration conflict error
    pub fn registration_conflict(rule_id: impl Into<String>) -> Self {
        Self::RegistrationConflict {
            rule_id: rule_id.into(),
        }
    }
}

/// Convenience constructors for document errors
impl DocumentError {
    /// Create a read failed error
    pub fn read_failed(path: impl Into<String>) -> Self {
        Self::ReadFailed { path: path.into() }
    }

    /// Create a parse failed error
    pub fn parse_failed(reason: impl Into<String>) -> Self {
        Self::ParseFailed {
            reason: reason.into(),
        }
    }

    /// Create a too large error
    pub fn too_large(size: usize, max_size: usize) -> Self {
        Self::TooLarge { size, max_size }
    }

    /// Create an invalid encoding error
    pub fn invalid_encoding(path: impl Into<String>) -> Self {
        Self::InvalidEncoding { path: path.into() }
    }
}

/// Error context extension trait for adding contextual information to errors
pub trait ErrorContext<T> {
    /// Add rule context to an error
    fn with_rule_context(self, rule_id: &str) -> Result<T>;

    /// Add document context to an error
    fn with_document_context(self, path: &str) -> Result<T>;

    /// Add plugin context to an error
    fn with_plugin_context(self, plugin_id: &str) -> Result<T>;

    /// Add configuration context to an error
    fn with_config_context(self, field: &str) -> Result<T>;
}

impl<T> ErrorContext<T> for std::result::Result<T, MdBookLintError> {
    fn with_rule_context(self, rule_id: &str) -> Result<T> {
        self.map_err(|e| match e {
            MdBookLintError::Rule { message, .. } => MdBookLintError::Rule {
                rule_id: rule_id.to_string(),
                message,
            },
            other => other,
        })
    }

    fn with_document_context(self, path: &str) -> Result<T> {
        self.map_err(|e| match e {
            MdBookLintError::Document(message) => {
                MdBookLintError::Document(format!("{path}: {message}"))
            }
            other => other,
        })
    }

    fn with_plugin_context(self, plugin_id: &str) -> Result<T> {
        self.map_err(|e| match e {
            MdBookLintError::Plugin(message) => {
                MdBookLintError::Plugin(format!("{plugin_id}: {message}"))
            }
            other => other,
        })
    }

    fn with_config_context(self, field: &str) -> Result<T> {
        self.map_err(|e| match e {
            MdBookLintError::Config(message) => {
                MdBookLintError::Config(format!("{field}: {message}"))
            }
            other => other,
        })
    }
}

/// Extension trait for converting specialized errors to MdBookLintError
pub trait IntoMdBookLintError<T> {
    /// Convert into a Result<T, MdBookLintError>
    fn into_mdbook_lint_error(self) -> Result<T>;
}

impl<T> IntoMdBookLintError<T> for std::result::Result<T, RuleError> {
    fn into_mdbook_lint_error(self) -> Result<T> {
        self.map_err(|e| match e {
            RuleError::NotFound { rule_id } => {
                MdBookLintError::rule_error(rule_id, "Rule not found")
            }
            RuleError::ExecutionFailed { message } => {
                MdBookLintError::rule_error("unknown", message)
            }
            RuleError::InvalidConfig { message } => MdBookLintError::config_error(message),
            RuleError::DependencyNotMet {
                rule_id,
                dependency,
            } => MdBookLintError::rule_error(rule_id, format!("Dependency not met: {dependency}")),
            RuleError::RegistrationConflict { rule_id } => {
                MdBookLintError::registry_error(format!("Rule already exists: {rule_id}"))
            }
        })
    }
}

impl<T> IntoMdBookLintError<T> for std::result::Result<T, DocumentError> {
    fn into_mdbook_lint_error(self) -> Result<T> {
        self.map_err(|e| match e {
            DocumentError::ReadFailed { path } => {
                MdBookLintError::document_error(format!("Failed to read: {path}"))
            }
            DocumentError::InvalidFormat => {
                MdBookLintError::document_error("Invalid document format")
            }
            DocumentError::TooLarge { size, max_size } => MdBookLintError::document_error(format!(
                "Document too large: {size} bytes (max: {max_size})"
            )),
            DocumentError::ParseFailed { reason } => {
                MdBookLintError::document_error(format!("Parse failed: {reason}"))
            }
            DocumentError::InvalidEncoding { path } => {
                MdBookLintError::document_error(format!("Invalid encoding: {path}"))
            }
        })
    }
}

impl<T> IntoMdBookLintError<T> for std::result::Result<T, ConfigError> {
    fn into_mdbook_lint_error(self) -> Result<T> {
        self.map_err(|e| match e {
            ConfigError::NotFound { path } => {
                MdBookLintError::config_error(format!("Configuration not found: {path}"))
            }
            ConfigError::InvalidFormat { message } => {
                MdBookLintError::config_error(format!("Invalid format: {message}"))
            }
            ConfigError::ValidationFailed { field, message } => {
                MdBookLintError::config_error(format!("Validation failed for {field}: {message}"))
            }
            ConfigError::UnsupportedVersion { version, supported } => {
                MdBookLintError::config_error(format!(
                    "Unsupported version: {version} (supported: {supported})"
                ))
            }
        })
    }
}

impl<T> IntoMdBookLintError<T> for std::result::Result<T, PluginError> {
    fn into_mdbook_lint_error(self) -> Result<T> {
        self.map_err(|e| match e {
            PluginError::NotFound { plugin_id } => {
                MdBookLintError::plugin_error(format!("Plugin not found: {plugin_id}"))
            }
            PluginError::LoadFailed { plugin_id, reason } => {
                MdBookLintError::plugin_error(format!("Failed to load {plugin_id}: {reason}"))
            }
            PluginError::InitializationFailed { plugin_id } => {
                MdBookLintError::plugin_error(format!("Initialization failed: {plugin_id}"))
            }
            PluginError::VersionIncompatible {
                plugin_id,
                version,
                required,
            } => MdBookLintError::plugin_error(format!(
                "Version incompatible: {plugin_id} version {version} (required: {required})"
            )),
        })
    }
}

// Compatibility layer for migration from anyhow
impl From<anyhow::Error> for MdBookLintError {
    fn from(err: anyhow::Error) -> Self {
        MdBookLintError::Document(err.to_string())
    }
}

// Note: anyhow already provides From<MdBookLintError> via its blanket impl for StdError
// So we don't need to implement From<MdBookLintError> for anyhow::Error

/// Compatibility alias for the old error name
pub type MdlntError = MdBookLintError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = MdBookLintError::parse_error(10, 5, "Invalid syntax");
        assert!(matches!(
            err,
            MdBookLintError::Parse {
                line: 10,
                column: 5,
                ..
            }
        ));
        assert!(err.to_string().contains("line 10, column 5"));
    }

    #[test]
    fn test_all_error_variants() {
        // Test Config error
        let config_err = MdBookLintError::config_error("Invalid config");
        assert!(matches!(config_err, MdBookLintError::Config(_)));

        // Test Rule error
        let rule_err = MdBookLintError::rule_error("MD001", "Rule failed");
        assert!(matches!(rule_err, MdBookLintError::Rule { .. }));

        // Test Plugin error
        let plugin_err = MdBookLintError::plugin_error("Plugin failed");
        assert!(matches!(plugin_err, MdBookLintError::Plugin(_)));

        // Test Document error
        let doc_err = MdBookLintError::document_error("Document error");
        assert!(matches!(doc_err, MdBookLintError::Document(_)));

        // Test Registry error
        let registry_err = MdBookLintError::registry_error("Registry error");
        assert!(matches!(registry_err, MdBookLintError::Registry(_)));
    }

    #[test]
    fn test_rule_error_variants() {
        let not_found = RuleError::not_found("MD999");
        assert!(matches!(not_found, RuleError::NotFound { .. }));
        assert!(not_found.to_string().contains("MD999"));

        let exec_failed = RuleError::execution_failed("Test execution failed");
        assert!(matches!(exec_failed, RuleError::ExecutionFailed { .. }));

        let invalid_config = RuleError::invalid_config("Invalid rule config");
        assert!(matches!(invalid_config, RuleError::InvalidConfig { .. }));

        let dep_not_met = RuleError::dependency_not_met("MD001", "MD002");
        assert!(matches!(dep_not_met, RuleError::DependencyNotMet { .. }));

        let reg_conflict = RuleError::registration_conflict("MD001");
        assert!(matches!(
            reg_conflict,
            RuleError::RegistrationConflict { .. }
        ));
    }

    #[test]
    fn test_document_error_variants() {
        let read_failed = DocumentError::read_failed("test.md");
        assert!(matches!(read_failed, DocumentError::ReadFailed { .. }));

        let parse_failed = DocumentError::parse_failed("Parse error");
        assert!(matches!(parse_failed, DocumentError::ParseFailed { .. }));

        let too_large = DocumentError::too_large(1000, 500);
        assert!(matches!(too_large, DocumentError::TooLarge { .. }));

        let invalid_encoding = DocumentError::invalid_encoding("test.md");
        assert!(matches!(
            invalid_encoding,
            DocumentError::InvalidEncoding { .. }
        ));
    }

    #[test]
    fn test_config_error_variants() {
        let not_found = ConfigError::NotFound {
            path: "config.toml".to_string(),
        };
        assert!(not_found.to_string().contains("config.toml"));

        let invalid_format = ConfigError::InvalidFormat {
            message: "Bad YAML".to_string(),
        };
        assert!(invalid_format.to_string().contains("Bad YAML"));

        let validation_failed = ConfigError::ValidationFailed {
            field: "rules".to_string(),
            message: "Invalid rule".to_string(),
        };
        assert!(validation_failed.to_string().contains("rules"));

        let unsupported_version = ConfigError::UnsupportedVersion {
            version: "2.0".to_string(),
            supported: "1.0-1.5".to_string(),
        };
        assert!(unsupported_version.to_string().contains("2.0"));
    }

    #[test]
    fn test_plugin_error_variants() {
        let not_found = PluginError::NotFound {
            plugin_id: "test-plugin".to_string(),
        };
        assert!(not_found.to_string().contains("test-plugin"));

        let load_failed = PluginError::LoadFailed {
            plugin_id: "test-plugin".to_string(),
            reason: "Missing file".to_string(),
        };
        assert!(load_failed.to_string().contains("Missing file"));

        let init_failed = PluginError::InitializationFailed {
            plugin_id: "test-plugin".to_string(),
        };
        assert!(init_failed.to_string().contains("test-plugin"));

        let version_incompatible = PluginError::VersionIncompatible {
            plugin_id: "test-plugin".to_string(),
            version: "2.0".to_string(),
            required: "1.0".to_string(),
        };
        assert!(version_incompatible.to_string().contains("2.0"));
    }

    #[test]
    fn test_error_context() {
        let result: Result<()> = Err(MdBookLintError::document_error("Something went wrong"));
        let with_context = result.with_document_context("test.md");

        assert!(with_context.is_err());
        assert!(with_context.unwrap_err().to_string().contains("test.md"));
    }

    #[test]
    fn test_all_error_contexts() {
        let base_err = MdBookLintError::document_error("Base error");

        let result: Result<()> = Err(MdBookLintError::document_error("Base error"));
        let with_rule = result.with_rule_context("MD001");
        assert!(with_rule.is_err());

        let result: Result<()> = Err(MdBookLintError::document_error("Base error"));
        let with_doc = result.with_document_context("test.md");
        assert!(with_doc.is_err());

        let result: Result<()> = Err(MdBookLintError::document_error("Base error"));
        let with_plugin = result.with_plugin_context("test-plugin");
        assert!(with_plugin.is_err());

        let result: Result<()> = Err(base_err);
        let with_config = result.with_config_context("config.toml");
        assert!(with_config.is_err());
    }

    #[test]
    fn test_rule_error_conversion() {
        let rule_err = RuleError::not_found("MD001");
        let result: std::result::Result<(), _> = Err(rule_err);
        let result = result.into_mdbook_lint_error();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("MD001"));
    }

    #[test]
    fn test_all_error_conversions() {
        // Document error conversion
        let doc_err = DocumentError::parse_failed("Parse failed");
        let result: std::result::Result<(), _> = Err(doc_err);
        let converted = result.into_mdbook_lint_error();
        assert!(converted.is_err());
        assert!(matches!(
            converted.unwrap_err(),
            MdBookLintError::Document(_)
        ));

        // Config error conversion
        let config_err = ConfigError::InvalidFormat {
            message: "Bad format".to_string(),
        };
        let result: std::result::Result<(), _> = Err(config_err);
        let converted = result.into_mdbook_lint_error();
        assert!(converted.is_err());
        assert!(matches!(converted.unwrap_err(), MdBookLintError::Config(_)));

        // Plugin error conversion
        let plugin_err = PluginError::NotFound {
            plugin_id: "missing".to_string(),
        };
        let result: std::result::Result<(), _> = Err(plugin_err);
        let converted = result.into_mdbook_lint_error();
        assert!(converted.is_err());
        assert!(matches!(converted.unwrap_err(), MdBookLintError::Plugin(_)));
    }

    #[test]
    fn test_anyhow_compatibility() {
        let anyhow_err = anyhow::anyhow!("Test error");
        let mdbook_lint_err: MdBookLintError = anyhow_err.into();
        // anyhow provides blanket From<E> impl for types implementing std::error::Error
        let back_to_anyhow = anyhow::Error::from(mdbook_lint_err);

        assert!(back_to_anyhow.to_string().contains("Test error"));
    }

    #[test]
    fn test_io_error_conversion() {
        use std::io::{Error, ErrorKind};

        let io_err = Error::new(ErrorKind::NotFound, "File not found");
        let mdbook_lint_err: MdBookLintError = io_err.into();

        assert!(matches!(mdbook_lint_err, MdBookLintError::Io(_)));
        assert!(mdbook_lint_err.to_string().contains("File not found"));
    }

    #[test]
    fn test_error_chain_context() {
        // Test chaining multiple contexts
        let base_err = MdBookLintError::parse_error(1, 1, "Parse error");
        let result: Result<()> = Err(base_err);

        let chained: Result<()> = result.with_document_context("test.md");

        assert!(chained.is_err());
        let error_string = chained.unwrap_err().to_string();
        assert!(
            error_string.contains("Parse error"),
            "Error should contain original message"
        );
    }

    #[test]
    fn test_error_source_chain() {
        use std::error::Error;

        let inner_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let mdbook_err: MdBookLintError = inner_err.into();

        // Test that the error source chain works
        assert!(mdbook_err.source().is_some());
        assert!(
            mdbook_err
                .source()
                .unwrap()
                .to_string()
                .contains("File not found")
        );
    }

    #[test]
    fn test_mdlnt_error_alias() {
        // Test that the MdlntError alias works
        let _err: MdlntError = MdBookLintError::document_error("Test");
        // This is mainly a compile-time test to ensure the alias exists
    }
}
