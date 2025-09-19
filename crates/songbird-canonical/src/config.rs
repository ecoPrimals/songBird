//! Canonical Configuration System for Songbird Ecosystem Ecosystem
//!
//! **MODERNIZED**: This configuration system has been refactored from a single 697-line
//! file into focused, maintainable modules. Each module handles a specific domain while
//! maintaining a unified interface.
//!
//! ## Module Organization
//!
//! - `orchestration`: Service discovery, load balancing, health monitoring, scaling
//! - `adapters`: Universal adapters for ecosystem primals (security_provider, compute_provider, storage_provider)
//! - `ai_first`: AI-First Citizen API compliance and human-AI collaboration
//! - `performance`: Zero-cost abstractions, memory, throughput, and latency optimization
//! - `environment`: Deployment settings, networking, logging, observability
//!
//! ## Benefits Benefits
//!
//! - **Maintainability**: Focused modules instead of monolithic configuration
//! - **Readability**: Clear separation of concerns
//! - **Testability**: Each module can be tested independently
//! - **Extensibility**: Easy to add new configuration domains

// Re-export the entire modular configuration system;
pub use self::config::*;

// Import the modular config system
mod config;

// Legacy compatibility - the main CanonicalConfig struct is still available
// with the same interface, but now backed by modular implementation
