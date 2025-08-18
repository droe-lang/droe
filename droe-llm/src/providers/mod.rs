pub mod ollama;

pub use ollama::{OllamaClient, OllamaError, InferenceParams};

// Simple API functions for unified binary (separate from gRPC clients)
pub mod simple {
    
    
    pub mod ollama {
        use crate::Result;
        
        pub async fn chat(model: &str, message: &str, _history: &[(String, String)]) -> Result<String> {
            // TODO: Implement actual Ollama integration
            Ok(format!("Ollama {} response to: {}", model, message))
        }
    }

    pub mod anthropic {
        use crate::Result;
        
        pub async fn chat(model: &str, message: &str, _history: &[(String, String)]) -> Result<String> {
            // TODO: Implement actual Anthropic integration
            Ok(format!("Anthropic {} response to: {}", model, message))
        }
    }

    pub mod openai {
        use crate::Result;
        
        pub async fn chat(model: &str, message: &str, _history: &[(String, String)]) -> Result<String> {
            // TODO: Implement actual OpenAI integration
            Ok(format!("OpenAI {} response to: {}", model, message))
        }
    }
}