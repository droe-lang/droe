//! Go language framework adapters
//! 
//! Supports multiple Go web frameworks:
//! - Fiber: Fast Express-inspired framework
//! - Gin: High-performance HTTP framework (TODO)
//! - Echo: Minimalist web framework (TODO)

pub mod fiber;

// Re-export adapters
pub use fiber::FiberAdapter;

// Future adapters can be added here:
// pub mod gin;
// pub mod echo;
// pub use gin::GinAdapter;
// pub use echo::EchoAdapter;