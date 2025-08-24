//! LSP server implementation for mdbook-lint
//!
//! This module provides a Language Server Protocol server for real-time markdown
//! linting in editors. It supports both general markdown linting and mdBook-specific
//! enhancements.
//!
//! This module is only available when the `lsp` feature is enabled.

use crate::config::Config;
use mdbook_lint_core::{Document, LintEngine, Severity, Violation, PluginRegistry};
use mdbook_lint_rulesets::{StandardRuleProvider, MdBookRuleProvider};
use std::collections::HashMap;
use std::path::PathBuf;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

/// The main LSP server implementation
pub struct MdBookLintServer {
    client: Client,
    engine: LintEngine,
    document_map: tokio::sync::RwLock<HashMap<Url, String>>,
    config: tokio::sync::RwLock<Config>,
}

impl MdBookLintServer {
    pub fn new(client: Client) -> Self {
        let mut registry = PluginRegistry::new();
        registry.register_provider(Box::new(StandardRuleProvider)).expect("Failed to register standard rules");
        registry.register_provider(Box::new(MdBookRuleProvider)).expect("Failed to register mdbook rules");
        let engine = registry.create_engine().expect("Failed to create engine");
        
        Self {
            client,
            engine,
            document_map: tokio::sync::RwLock::new(HashMap::new()),
            config: tokio::sync::RwLock::new(Config::default()),
        }
    }

    /// Lint a document and convert violations to LSP diagnostics
    async fn lint_document(&self, uri: &Url, text: &str) -> Vec<Diagnostic> {
        let path = uri
            .to_file_path()
            .unwrap_or_else(|_| PathBuf::from("untitled.md"));

        let document = match Document::new(text.to_string(), path) {
            Ok(doc) => doc,
            Err(_) => return Vec::new(),
        };

        let config = self.config.read().await;
        let violations = match self
            .engine
            .lint_document_with_config(&document, &config.core)
        {
            Ok(violations) => violations,
            Err(_) => return Vec::new(),
        };

        violations
            .into_iter()
            .map(|violation| self.violation_to_diagnostic(violation))
            .collect()
    }

    /// Convert a mdbook-lint violation to an LSP diagnostic
    fn violation_to_diagnostic(&self, violation: Violation) -> Diagnostic {
        let severity = match violation.severity {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
            Severity::Info => DiagnosticSeverity::INFORMATION,
        };

        let range = Range {
            start: Position {
                line: (violation.line.saturating_sub(1)) as u32,
                character: (violation.column.saturating_sub(1)) as u32,
            },
            end: Position {
                line: (violation.line.saturating_sub(1)) as u32,
                character: violation.column as u32, // End one character after start for simplicity
            },
        };

        Diagnostic {
            range,
            severity: Some(severity),
            code: Some(NumberOrString::String(violation.rule_id.clone())),
            code_description: None,
            source: Some("mdbook-lint".to_string()),
            message: violation.message,
            related_information: None,
            tags: None,
            data: None,
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for MdBookLintServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // Detect if we're in an mdBook project and load config
        let (is_mdbook_project, config_loaded) = if let Some(root_uri) = &params.root_uri {
            if let Ok(root_path) = root_uri.to_file_path() {
                let is_mdbook =
                    root_path.join("book.toml").exists() || root_path.join("SUMMARY.md").exists();

                // Try to load .mdbook-lint.toml config if we're in an mdBook project
                let mut config_loaded = false;
                if is_mdbook {
                    let config_path = root_path.join(".mdbook-lint.toml");
                    if config_path.exists()
                        && let Ok(config_content) = std::fs::read_to_string(&config_path)
                        && let Ok(config) = Config::from_toml_str(&config_content)
                    {
                        *self.config.write().await = config;
                        config_loaded = true;
                        self.client
                            .log_message(
                                MessageType::INFO,
                                format!("Loaded config from {}", config_path.display()),
                            )
                            .await;
                    }
                }
                (is_mdbook, config_loaded)
            } else {
                (false, false)
            }
        } else {
            (false, false)
        };

        // Log initialization info
        let message = match (is_mdbook_project, config_loaded) {
            (true, true) => "mdbook-lint LSP initialized for mdBook project with custom config",
            (true, false) => "mdbook-lint LSP initialized for mdBook project with default config",
            (false, _) => "mdbook-lint LSP initialized for markdown project",
        };
        self.client.log_message(MessageType::INFO, message).await;

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("mdbook-lint".to_string()),
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                    },
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "mdbook-lint".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "mdbook-lint LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        // Store document content
        self.document_map
            .write()
            .await
            .insert(uri.clone(), text.clone());

        // Lint and publish diagnostics
        let diagnostics = self.lint_document(&uri, &text).await;

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        // Get the full new text (we use FULL sync mode)
        if let Some(change) = params.content_changes.into_iter().next() {
            let text = change.text;

            // Store updated content
            self.document_map
                .write()
                .await
                .insert(uri.clone(), text.clone());

            // Lint and publish diagnostics
            let diagnostics = self.lint_document(&uri, &text).await;

            self.client
                .publish_diagnostics(uri, diagnostics, None)
                .await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        // Re-lint on save to ensure consistency
        let uri = params.text_document.uri;

        if let Some(text) = self.document_map.read().await.get(&uri) {
            let diagnostics = self.lint_document(&uri, text).await;

            self.client
                .publish_diagnostics(uri, diagnostics, None)
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        // Remove from document map and clear diagnostics
        self.document_map
            .write()
            .await
            .remove(&params.text_document.uri);

        self.client
            .publish_diagnostics(params.text_document.uri, Vec::new(), None)
            .await;
    }

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult> {
        let uri = params.text_document.uri;

        if let Some(text) = self.document_map.read().await.get(&uri) {
            let diagnostics = self.lint_document(&uri, text).await;

            Ok(DocumentDiagnosticReportResult::Report(
                DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                    related_documents: None,
                    full_document_diagnostic_report: FullDocumentDiagnosticReport {
                        result_id: None,
                        items: diagnostics,
                    },
                }),
            ))
        } else {
            Ok(DocumentDiagnosticReportResult::Report(
                DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                    related_documents: None,
                    full_document_diagnostic_report: FullDocumentDiagnosticReport {
                        result_id: None,
                        items: Vec::new(),
                    },
                }),
            ))
        }
    }
}

/// Run the LSP server
pub async fn run_lsp_server(_stdio: bool, port: Option<u16>) -> mdbook_lint_core::Result<()> {
    if let Some(port) = port {
        // TCP mode
        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await?;
        eprintln!("mdbook-lint LSP server listening on port {port}");

        let (stream, _) = listener.accept().await?;
        let (read, write) = tokio::io::split(stream);

        let (service, socket) = LspService::new(MdBookLintServer::new);
        Server::new(read, write, socket).serve(service).await;
    } else {
        // stdio mode (default)
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        let (service, socket) = LspService::new(MdBookLintServer::new);
        Server::new(stdin, stdout, socket).serve(service).await;
    }

    Ok(())
}
