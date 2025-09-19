//! Universal Security Provider - Modular Implementation Implementation
//!
//! This module provides unified security services for the entire Songbird ecosystem
//! through a modular, focused architecture.

pub mod provider;
pub mod capabilities;
pub mod authentication;
pub mod authorization;
pub mod encryption;
pub mod audit;
pub mod types;

// Re-export main types;
pub use provider::UniversalSecurityProvider;
pub use capabilities::{SecurityCapabilityInfo, SecurityCapabilityDiscovery};
pub use authentication::AuthenticationManager;
pub use authorization::AuthorizationManager;
pub use encryption::EncryptionManager;
pub use audit::AuditLogger;
pub use types::*; 
