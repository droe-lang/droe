//! Droe Language Server Protocol Implementation

use droe_compiler::Compiler;
use std::collections::HashMap;
use tokio::io::{stdin, stdout};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

pub struct DroeLanguageServer {
    client: Client,
    compiler: Compiler,
    document_map: tokio::sync::RwLock<HashMap<String, String>>,
}

impl DroeLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            compiler: Compiler::new(),
            document_map: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
    
    async fn validate_document(&self, uri: &Url, text: &str) {
        let diagnostics = self.compiler.lint(text);
        
        let lsp_diagnostics: Vec<tower_lsp::lsp_types::Diagnostic> = diagnostics
            .into_iter()
            .map(|d| tower_lsp::lsp_types::Diagnostic {
                range: Range {
                    start: Position {
                        line: d.line as u32,
                        character: d.character as u32,
                    },
                    end: Position {
                        line: d.line as u32,
                        character: (d.character + 1) as u32, // Simple end position
                    },
                },
                severity: Some(match d.severity {
                    droe_compiler::diagnostics::Severity::Error => DiagnosticSeverity::ERROR,
                    droe_compiler::diagnostics::Severity::Warning => DiagnosticSeverity::WARNING,
                    droe_compiler::diagnostics::Severity::Information => DiagnosticSeverity::INFORMATION,
                    droe_compiler::diagnostics::Severity::Hint => DiagnosticSeverity::HINT,
                }),
                code: None,
                code_description: None,
                source: Some(d.source),
                message: d.message,
                related_information: None,
                tags: None,
                data: None,
            })
            .collect();
        
        self.client
            .publish_diagnostics(uri.clone(), lsp_diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for DroeLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "droe-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("droe".to_string()),
                        inter_file_dependencies: true,
                        workspace_diagnostics: false,
                        work_done_progress_options: Default::default(),
                    },
                )),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Droe Language Server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        
        self.document_map
            .write()
            .await
            .insert(uri.to_string(), text.clone());
        
        self.validate_document(&uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        
        if let Some(change) = params.content_changes.into_iter().next() {
            let text = change.text;
            
            self.document_map
                .write()
                .await
                .insert(uri.to_string(), text.clone());
            
            self.validate_document(&uri, &text).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if let Some(text) = params.text {
            self.validate_document(&params.text_document.uri, &text).await;
        }
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let keywords = vec![
            "module", "end", "data", "action", "task", "when", "then", "otherwise",
            "while", "for", "each", "in", "give", "display", "set", "to", "is",
            "include", "from", "true", "false", "and", "or", "not", "equals",
        ];

        let completions: Vec<CompletionItem> = keywords
            .into_iter()
            .map(|keyword| CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Droe keyword".to_string()),
                ..CompletionItem::default()
            })
            .collect();

        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn hover(&self, _params: HoverParams) -> Result<Option<Hover>> {
        // TODO: Implement hover information
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(
                "Droe Language Element".to_string(),
            )),
            range: None,
        }))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        
        if let Some(text) = self.document_map.read().await.get(&uri.to_string()) {
            match self.compiler.format(text) {
                Ok(formatted) => {
                    if formatted != *text {
                        return Ok(Some(vec![TextEdit {
                            range: Range {
                                start: Position::new(0, 0),
                                end: Position::new(u32::MAX, u32::MAX),
                            },
                            new_text: formatted,
                        }]));
                    }
                }
                Err(_) => {
                    // Format failed, return no changes
                }
            }
        }
        
        Ok(None)
    }
}

pub async fn run_server() -> anyhow::Result<()> {
    tracing::info!("Starting Droe Language Server");

    let (service, socket) = LspService::build(|client| DroeLanguageServer::new(client))
        .finish();

    Server::new(stdin(), stdout(), socket)
        .serve(service)
        .await;

    Ok(())
}