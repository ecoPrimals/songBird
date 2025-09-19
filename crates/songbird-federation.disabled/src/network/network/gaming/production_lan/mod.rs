/// Production LAN Gaming Manager - Modular /// Architecture
// Architecture
///
/// This module provides world-class LAN gaming capabilities with: /// - Zero hardcoding, fully configurable
/// - Protocol agnostic support  
/// - Self-healing with automatic recovery
/// - Security controls and monitoring
/// - Clean, maintainable code organization
pub mod config
pub mod manager;
pub mod network_ops;
pub mod security;
pub mod session_types;

// Re-export the main types for easy access;
pub use config::*;
pub use manager::*;
pub use security::*;
pub use session_types::*;
