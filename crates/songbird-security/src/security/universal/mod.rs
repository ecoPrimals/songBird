pub mod crypto;
pub mod manager;
/// Universal Security Module
///
/// Provides intelligent modularization of universal security functionality
///
/// ## Architecture
/// - `types`: Core data structures and configuration types
/// - `crypto`: Lightweight encryption and key management for tunnel coordination
/// - `manager`: High-level security management and orchestration
///
/// This module demonstrates smart refactoring by splitting a 896-line monolithic file
/// into focused, single-responsibility modules for better maintainability.
///
/// ## Principles
/// - Zero Trust by Default
/// - Privacy by Design  
/// - Security Without Complexity
/// - Universal Access to Protection
// Submodules organized by responsibility
pub mod types;

// Re-exports for backward compatibility and convenience
pub use crypto::*;
pub use manager::*;
pub use types::*;

// Convenience type aliases
pub type SecurityManager = UniversalSecurityManager;
pub type CryptoManager = LightweightTunnelCrypto;
pub type ProviderManager = SecurityProviderManager;
