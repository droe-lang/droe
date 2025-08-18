//! Java language framework adapters
//! 
//! Supports multiple Java web frameworks:
//! - Spring Boot: Enterprise framework (TODO)
//! - Quarkus: Cloud-native framework (TODO)
//! - Micronaut: Microservices framework (TODO)

pub mod spring;

// Re-export adapters
pub use spring::SpringAdapter;