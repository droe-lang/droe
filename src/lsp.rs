use anyhow::Result;
use tower_lsp::{LspService, Server};
use tower_lsp::jsonrpc;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use console::style;
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct DroeLsp {
    client: Client,
    documents: RwLock<HashMap<Url, String>>,
}

impl DroeLsp {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: RwLock::new(HashMap::new()),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for DroeLsp {
    async fn initialize(&self, _: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), " ".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("droe".to_string()),
                        inter_file_dependencies: true,
                        workspace_diagnostics: true,
                        ..Default::default()
                    }
                )),
                document_formatting_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Droe LSP server initialized!")
            .await;
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;
        
        self.documents.write().await.insert(uri.clone(), content);
        
        // Perform initial diagnostics
        self.validate_document(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        
        if let Some(change) = params.content_changes.into_iter().next() {
            self.documents.write().await.insert(uri.clone(), change.text);
            self.validate_document(&uri).await;
        }
    }

    async fn completion(&self, params: CompletionParams) -> jsonrpc::Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        
        let documents = self.documents.read().await;
        if let Some(content) = documents.get(&uri) {
            let completions = self.get_completions(content, position).await;
            return Ok(Some(CompletionResponse::Array(completions)));
        }
        
        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        
        let documents = self.documents.read().await;
        if let Some(content) = documents.get(&uri) {
            if let Some(hover_info) = self.get_hover_info(content, position).await {
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: hover_info,
                    }),
                    range: None,
                }));
            }
        }
        
        Ok(None)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> jsonrpc::Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        
        let documents = self.documents.read().await;
        if let Some(content) = documents.get(&uri) {
            if let Ok(formatted) = self.format_document(content).await {
                return Ok(Some(vec![TextEdit {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: u32::MAX, character: u32::MAX },
                    },
                    new_text: formatted,
                }]));
            }
        }
        
        Ok(None)
    }
}

impl DroeLsp {
    async fn validate_document(&self, uri: &Url) {
        let documents = self.documents.read().await;
        if let Some(content) = documents.get(uri) {
            let diagnostics = self.get_diagnostics(content).await;
            
            self.client
                .publish_diagnostics(uri.clone(), diagnostics, None)
                .await;
        }
    }
    
    async fn get_diagnostics(&self, content: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        
        // Use droe-compiler for syntax checking
        if let Err(errors) = droe_compiler::parse_and_validate(content) {
            for error in errors {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: error.line as u32, character: error.column as u32 },
                        end: Position { line: error.line as u32, character: error.column as u32 + error.length as u32 },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    source: Some("droe".to_string()),
                    message: error.message,
                    ..Default::default()
                });
            }
        }
        
        diagnostics
    }
    
    async fn get_completions(&self, _content: &str, _position: Position) -> Vec<CompletionItem> {
        let mut completions = Vec::new();
        
        // Keywords
        let keywords = ["Display", "action", "serve", "route", "layout", "screen", "if", "while", "for"];
        for keyword in keywords {
            completions.push(CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(format!("Droe keyword: {}", keyword)),
                ..Default::default()
            });
        }
        
        // Functions
        let functions = ["format", "length", "substring", "contains"];
        for func in functions {
            completions.push(CompletionItem {
                label: func.to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(format!("Droe function: {}", func)),
                ..Default::default()
            });
        }
        
        completions
    }
    
    async fn get_hover_info(&self, content: &str, position: Position) -> Option<String> {
        // Extract word at position and provide hover information
        let lines: Vec<&str> = content.lines().collect();
        if let Some(line) = lines.get(position.line as usize) {
            // Simple word extraction logic
            let chars: Vec<char> = line.chars().collect();
            if let Some(start) = self.find_word_start(&chars, position.character as usize) {
                if let Some(end) = self.find_word_end(&chars, position.character as usize) {
                    let word: String = chars[start..end].iter().collect();
                    return self.get_word_documentation(&word);
                }
            }
        }
        None
    }
    
    fn find_word_start(&self, chars: &[char], pos: usize) -> Option<usize> {
        for i in (0..pos).rev() {
            if !chars[i].is_alphanumeric() && chars[i] != '_' {
                return Some(i + 1);
            }
        }
        Some(0)
    }
    
    fn find_word_end(&self, chars: &[char], pos: usize) -> Option<usize> {
        for i in pos..chars.len() {
            if !chars[i].is_alphanumeric() && chars[i] != '_' {
                return Some(i);
            }
        }
        Some(chars.len())
    }
    
    fn get_word_documentation(&self, word: &str) -> Option<String> {
        match word {
            "Display" => Some("**Display** - Output text or values to the console".to_string()),
            "action" => Some("**action** - Define a reusable action block".to_string()),
            "serve" => Some("**serve** - Start a web server with routes".to_string()),
            "route" => Some("**route** - Define an HTTP route".to_string()),
            _ => None,
        }
    }
    
    async fn format_document(&self, content: &str) -> Result<String> {
        // Use droe-compiler for formatting
        droe_compiler::format_code(content)
    }
}

pub async fn start_server(mode: &str, port: u16) -> Result<()> {
    match mode {
        "stdio" => {
            println!("{} Starting Droe LSP server (stdio mode)...", style("[INFO]").cyan());
            
            let stdin = tokio::io::stdin();
            let stdout = tokio::io::stdout();
            
            let (service, socket) = LspService::new(|client| DroeLsp::new(client));
            Server::new(stdin, stdout, socket).serve(service).await;
        }
        "tcp" => {
            println!("{} Starting Droe LSP server on port {}...", style("[INFO]").cyan(), port);
            
            let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
            
            loop {
                let (stream, _) = listener.accept().await?;
                let (read, write) = tokio::io::split(stream);
                
                let (service, socket) = LspService::new(|client| DroeLsp::new(client));
                
                tokio::spawn(async move {
                    Server::new(read, write, socket).serve(service).await;
                });
            }
        }
        _ => {
            return Err(anyhow::anyhow!("Unsupported LSP mode: {}", mode));
        }
    }
    
    Ok(())
}