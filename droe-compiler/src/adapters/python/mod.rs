//! Python language framework adapters
//! 
//! Supports multiple Python web frameworks:
//! - FastAPI: Modern, fast web framework (TODO)
//! - Django: Full-featured web framework (TODO)
//! - Flask: Lightweight WSGI framework (TODO)

pub mod fastapi;

// Re-export adapters
pub use fastapi::FastAPIAdapter;