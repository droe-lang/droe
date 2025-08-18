//! Node.js language framework adapters
//! 
//! Supports multiple Node.js web frameworks:
//! - Fastify: Fast and low overhead framework (TODO)
//! - Express: Standard web framework (TODO)
//! - NestJS: Enterprise-grade framework (TODO)

pub mod fastify;

// Re-export adapters
pub use fastify::FastifyAdapter;

// Convenience constructors
impl FastifyAdapter {
    pub fn default() -> Result<Self, String> {
        Self::new()
    }
}