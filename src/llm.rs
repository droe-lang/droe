use anyhow::Result;
use console::style;
use dialoguer::{Input, Select};
use std::io::{self, Write};

pub async fn start_chat(provider: &str, model: Option<&String>) -> Result<()> {
    println!("{} Starting LLM chat mode with provider: {}", 
        style("[INFO]").cyan(), 
        provider
    );
    
    let model_name = match model {
        Some(m) => m.clone(),
        None => select_model(provider).await?,
    };
    
    println!("{} Using model: {}", style("[INFO]").yellow(), model_name);
    println!("{} Type 'exit' to quit, 'help' for commands\n", style("[TIP]").blue());
    
    let mut chat_session = ChatSession::new(provider, &model_name).await?;
    
    loop {
        print!("{} ", style("You:").bold().green());
        io::stdout().flush()?;
        
        let input: String = Input::new().interact_text()?;
        
        match input.trim() {
            "exit" | "quit" => {
                println!("{} Goodbye!", style("[INFO]").cyan());
                break;
            }
            "help" => {
                show_help();
                continue;
            }
            "clear" => {
                chat_session.clear_history();
                println!("{} Chat history cleared!", style("[INFO]").yellow());
                continue;
            }
            "" => continue,
            _ => {}
        }
        
        print!("{} ", style("Droe:").bold().blue());
        io::stdout().flush()?;
        
        match chat_session.send_message(&input).await {
            Ok(response) => {
                println!("{}", response);
            }
            Err(e) => {
                println!("{} Error: {}", style("[ERROR]").red(), e);
            }
        }
        
        println!(); // Add spacing
    }
    
    Ok(())
}

struct ChatSession {
    provider: String,
    model: String,
    history: Vec<(String, String)>, // (user, assistant) pairs
}

impl ChatSession {
    async fn new(provider: &str, model: &str) -> Result<Self> {
        Ok(Self {
            provider: provider.to_string(),
            model: model.to_string(),
            history: Vec::new(),
        })
    }
    
    async fn send_message(&mut self, message: &str) -> Result<String> {
        let response = match self.provider.as_str() {
            "ollama" => self.call_ollama(message).await?,
            "anthropic" => self.call_anthropic(message).await?,
            "openai" => self.call_openai(message).await?,
            _ => return Err(anyhow::anyhow!("Unsupported provider: {}", self.provider)),
        };
        
        self.history.push((message.to_string(), response.clone()));
        Ok(response)
    }
    
    fn clear_history(&mut self) {
        self.history.clear();
    }
    
    async fn call_ollama(&self, message: &str) -> Result<String> {
        // Use droe-llm simple providers for Ollama integration
        match droe_llm::providers::simple::ollama::chat(&self.model, message, &self.history).await {
            Ok(response) => Ok(response),
            Err(e) => Err(anyhow::anyhow!("Ollama error: {}", e)),
        }
    }
    
    async fn call_anthropic(&self, message: &str) -> Result<String> {
        // Use droe-llm simple providers for Anthropic integration
        match droe_llm::providers::simple::anthropic::chat(&self.model, message, &self.history).await {
            Ok(response) => Ok(response),
            Err(e) => Err(anyhow::anyhow!("Anthropic error: {}", e)),
        }
    }
    
    async fn call_openai(&self, message: &str) -> Result<String> {
        // Use droe-llm simple providers for OpenAI integration
        match droe_llm::providers::simple::openai::chat(&self.model, message, &self.history).await {
            Ok(response) => Ok(response),
            Err(e) => Err(anyhow::anyhow!("OpenAI error: {}", e)),
        }
    }
}

async fn select_model(provider: &str) -> Result<String> {
    let models = match provider {
        "ollama" => vec![
            "llama3.2:latest",
            "codellama:latest", 
            "mistral:latest",
            "deepseek-coder:latest",
        ],
        "anthropic" => vec![
            "claude-3-5-sonnet-20241022",
            "claude-3-haiku-20240307",
            "claude-3-opus-20240229",
        ],
        "openai" => vec![
            "gpt-4",
            "gpt-4-turbo",
            "gpt-3.5-turbo",
            "o1-preview",
        ],
        _ => return Err(anyhow::anyhow!("Unknown provider: {}", provider)),
    };
    
    let selection = Select::new()
        .with_prompt("Select a model")
        .default(0)
        .items(&models)
        .interact()?;
    
    Ok(models[selection].to_string())
}

fn show_help() {
    println!("\n{}", style("=== Droe LLM Chat Commands ===").bold());
    println!("{}: Send a message to the LLM", style("<message>").green());
    println!("{}: Show this help", style("help").yellow());
    println!("{}: Clear chat history", style("clear").yellow());
    println!("{}: Exit chat mode", style("exit").red());
    println!("\n{}", style("=== Example Usage ===").bold());
    println!("- Help me write a Droe program that creates a web API");
    println!("- Convert this Python code to Droe syntax");
    println!("- Explain how to use actions in Droe");
    println!();
}

pub async fn start_jsonrpc_server(port: u16, provider: &str) -> Result<()> {
    println!("{} Starting JSON-RPC LLM server on port {} with provider {}...", 
        style("[INFO]").cyan(), 
        port, 
        provider
    );
    
    // Start the JSON-RPC server using droe-llm
    let llm_service = droe_llm::LLMService::new(droe_llm::LLMConfig::default());
    let jsonrpc_server = droe_llm::JsonRpcServer::new(llm_service, port);
    
    println!("{} JSON-RPC LLM server started! VSCode extension can connect.", 
        style("[SUCCESS]").green()
    );
    
    // Start the server
    jsonrpc_server.start().await
        .map_err(|e| anyhow::anyhow!("JSON-RPC server error: {}", e))?;
    
    Ok(())
}