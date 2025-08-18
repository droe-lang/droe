use anyhow::Result;
use console::style;
use tokio::net::TcpListener;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonRequest {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonResponse {
    pub id: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub struct DroeDaemon {
    llm_enabled: bool,
    llm_provider: String,
    lsp_server: Option<Arc<RwLock<()>>>, // Placeholder for LSP server
}

impl DroeDaemon {
    pub fn new(llm_enabled: bool, llm_provider: String) -> Self {
        Self {
            llm_enabled,
            llm_provider,
            lsp_server: None,
        }
    }
    
    pub async fn handle_request(&self, request: DaemonRequest) -> DaemonResponse {
        match request.method.as_str() {
            "compile" => self.handle_compile(request).await,
            "format" => self.handle_format(request).await,
            "lint" => self.handle_lint(request).await,
            "chat" => self.handle_chat(request).await,
            "lsp" => self.handle_lsp(request).await,
            "vm.run" => self.handle_vm_run(request).await,
            "project.init" => self.handle_project_init(request).await,
            _ => DaemonResponse {
                id: request.id,
                result: None,
                error: Some(format!("Unknown method: {}", request.method)),
            },
        }
    }
    
    async fn handle_compile(&self, request: DaemonRequest) -> DaemonResponse {
        // Extract compile parameters
        let params = request.params;
        let input = params.get("input").and_then(|v| v.as_str()).unwrap_or("");
        let target = params.get("target").and_then(|v| v.as_str()).unwrap_or("wasm");
        let opt_level = params.get("opt_level").and_then(|v| v.as_u64()).unwrap_or(2) as u8;
        
        match crate::compiler::compile_file(
            &std::path::PathBuf::from(input),
            None,
            target,
            opt_level,
        ).await {
            Ok(_) => DaemonResponse {
                id: request.id,
                result: Some(serde_json::json!({"status": "success"})),
                error: None,
            },
            Err(e) => DaemonResponse {
                id: request.id,
                result: None,
                error: Some(e.to_string()),
            },
        }
    }
    
    async fn handle_format(&self, request: DaemonRequest) -> DaemonResponse {
        let params = request.params;
        let path = params.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let check = params.get("check").and_then(|v| v.as_bool()).unwrap_or(false);
        
        match crate::cli::format_path(&std::path::PathBuf::from(path), check).await {
            Ok(_) => DaemonResponse {
                id: request.id,
                result: Some(serde_json::json!({"status": "success"})),
                error: None,
            },
            Err(e) => DaemonResponse {
                id: request.id,
                result: None,
                error: Some(e.to_string()),
            },
        }
    }
    
    async fn handle_lint(&self, request: DaemonRequest) -> DaemonResponse {
        let params = request.params;
        let path = params.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let fix = params.get("fix").and_then(|v| v.as_bool()).unwrap_or(false);
        
        match crate::cli::lint_path(&std::path::PathBuf::from(path), fix).await {
            Ok(_) => DaemonResponse {
                id: request.id,
                result: Some(serde_json::json!({"status": "success"})),
                error: None,
            },
            Err(e) => DaemonResponse {
                id: request.id,
                result: None,
                error: Some(e.to_string()),
            },
        }
    }
    
    async fn handle_chat(&self, request: DaemonRequest) -> DaemonResponse {
        if !self.llm_enabled {
            return DaemonResponse {
                id: request.id,
                result: None,
                error: Some("LLM integration not enabled".to_string()),
            };
        }
        
        let params = request.params;
        let message = params.get("message").and_then(|v| v.as_str()).unwrap_or("");
        let model = params.get("model").and_then(|v| v.as_str());
        
        // Use LLM integration
        match self.call_llm(message, model).await {
            Ok(response) => DaemonResponse {
                id: request.id,
                result: Some(serde_json::json!({"response": response})),
                error: None,
            },
            Err(e) => DaemonResponse {
                id: request.id,
                result: None,
                error: Some(e.to_string()),
            },
        }
    }
    
    async fn handle_lsp(&self, request: DaemonRequest) -> DaemonResponse {
        let params = request.params;
        let method = params.get("method").and_then(|v| v.as_str()).unwrap_or("");
        
        // Forward LSP requests to embedded LSP server
        match self.forward_to_lsp(method, &params).await {
            Ok(result) => DaemonResponse {
                id: request.id,
                result: Some(result),
                error: None,
            },
            Err(e) => DaemonResponse {
                id: request.id,
                result: None,
                error: Some(e.to_string()),
            },
        }
    }
    
    async fn handle_vm_run(&self, request: DaemonRequest) -> DaemonResponse {
        let params = request.params;
        let file = params.get("file").and_then(|v| v.as_str()).unwrap_or("");
        let function = params.get("function").and_then(|v| v.as_str()).unwrap_or("main");
        let args: Vec<String> = params.get("args").and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
            .unwrap_or_else(Vec::new);
        
        match crate::vm::run_wasm(&std::path::PathBuf::from(file), function, &args).await {
            Ok(_) => DaemonResponse {
                id: request.id,
                result: Some(serde_json::json!({"status": "success"})),
                error: None,
            },
            Err(e) => DaemonResponse {
                id: request.id,
                result: None,
                error: Some(e.to_string()),
            },
        }
    }
    
    async fn handle_project_init(&self, request: DaemonRequest) -> DaemonResponse {
        let params = request.params;
        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("my-project");
        let template = params.get("template").and_then(|v| v.as_str()).unwrap_or("basic");
        
        match crate::cli::init_project(name, template).await {
            Ok(_) => DaemonResponse {
                id: request.id,
                result: Some(serde_json::json!({"status": "success"})),
                error: None,
            },
            Err(e) => DaemonResponse {
                id: request.id,
                result: None,
                error: Some(e.to_string()),
            },
        }
    }
    
    async fn call_llm(&self, message: &str, model: Option<&str>) -> Result<String> {
        // Use droe-llm for LLM integration
        let model_name = model.unwrap_or("default");
        
        match self.llm_provider.as_str() {
            "ollama" => droe_llm::providers::simple::ollama::chat(model_name, message, &[]).await
                .map_err(|e| anyhow::anyhow!("Ollama error: {}", e)),
            "anthropic" => droe_llm::providers::simple::anthropic::chat(model_name, message, &[]).await
                .map_err(|e| anyhow::anyhow!("Anthropic error: {}", e)),
            "openai" => droe_llm::providers::simple::openai::chat(model_name, message, &[]).await
                .map_err(|e| anyhow::anyhow!("OpenAI error: {}", e)),
            _ => Err(anyhow::anyhow!("Unsupported LLM provider: {}", self.llm_provider)),
        }
    }
    
    async fn forward_to_lsp(&self, method: &str, _params: &serde_json::Value) -> Result<serde_json::Value> {
        // Forward to embedded LSP server
        // This would integrate with the LSP module
        Ok(serde_json::json!({"forwarded": true, "method": method}))
    }
}

pub async fn start_daemon(port: u16, llm_enabled: bool, llm_provider: &str) -> Result<()> {
    println!("{} Starting Droe daemon on port {}...", 
        style("[INFO]").cyan(), 
        port
    );
    
    if llm_enabled {
        println!("{} LLM integration enabled (provider: {})", 
            style("[INFO]").yellow(), 
            llm_provider
        );
    }
    
    let daemon = Arc::new(DroeDaemon::new(llm_enabled, llm_provider.to_string()));
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    
    println!("{} Daemon ready! Listening for requests...", style("[SUCCESS]").green());
    
    loop {
        let (socket, addr) = listener.accept().await?;
        let daemon = Arc::clone(&daemon);
        
        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            
            loop {
                let n = match socket.readable().await {
                    Ok(_) => {
                        match socket.try_read(&mut buf) {
                            Ok(n) if n == 0 => break,
                            Ok(n) => n,
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                            Err(_) => break,
                        }
                    }
                    Err(_) => break,
                };
                
                let request_str = String::from_utf8_lossy(&buf[..n]);
                
                if let Ok(request) = serde_json::from_str::<DaemonRequest>(&request_str) {
                    let response = daemon.handle_request(request).await;
                    
                    if let Ok(response_str) = serde_json::to_string(&response) {
                        socket.writable().await.ok();
                        socket.try_write(response_str.as_bytes()).ok();
                    }
                }
            }
            
            println!("{} Client {} disconnected", style("[INFO]").blue(), addr);
        });
    }
}