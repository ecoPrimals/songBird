/// Production LAN Gaming Manager - Modular Architecture
/// 
/// This module provides world-class LAN gaming capabilities with:
/// - Zero hardcoding, fully configurable
/// - Protocol agnostic support  
/// - Self-healing with automatic recovery
/// - Security controls and monitoring
/// - Clean, maintainable code organization
pub mod config;
pub mod session_types;
pub mod network_ops;
pub mod security;
pub mod manager;

// Re-export the main types for easy access
pub use config::*;
pub use session_types::*;
pub use security::*;
pub use manager::*; 