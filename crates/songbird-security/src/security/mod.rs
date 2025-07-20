//! Security Module
//!
//! Provides authentication, authorization, and security features

pub mod audit;
pub mod authentication;
pub mod beardog;
pub mod core;
pub mod encryption;
pub mod hardening;
pub mod oauth;
pub mod providers;
pub mod types;
pub mod universal_security;
pub mod zero_trust_middleware;

// Re-export main types and managers from core
pub use core::*;

// Re-export specific modules for backwards compatibility
pub use audit::*;
pub use authentication::*;
pub use beardog::*;
pub use encryption::*;
pub use hardening::*;
pub use oauth::*;
// Note: providers are available via specific imports rather than glob
pub use types::*;
pub use universal_security::*;
pub use zero_trust_middleware::ZeroTrustMiddleware;
