//! # Songbird Security Module
//!
//! This crate provides comprehensive security functionality for the Songbird ecosystem,
//! including authentication, authorization, encryption, and integration with external
//! security services like BearDog.
//!
//! ## Features
//!
//! - **Universal Security Integration**: Capability-based discovery of security services
//! - **BearDog Integration**: Enterprise-grade security with BearDog primal
//! - **Authentication & Authorization**: Token-based auth with role-based access control
//! - **Encryption Services**: Data encryption with key management
//! - **Universal Access**: Family-friendly security for all users
//! - **Zero Trust Middleware**: Comprehensive security middleware
//! - **Firewall & Hardening**: Production security hardening
//!
//! ## Usage
//!
//! ```rust,no_run
//! use songbird_security::{UniversalSecurityProvider, SecurityConfig};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize security with BearDog integration
//! let config = SecurityConfig::default();
//! let primal_registry = Arc::new(songbird_universal_primals::registry::UniversalPrimalRegistry::new());
//! let security = UniversalSecurityProvider::new(primal_registry, config);
//!
//! // Authenticate user
//! let token = security.authenticate("user", "password").await?;
//! println!("Authentication successful: {:?}", token);
//! # Ok(())
//! # }
//! ```

pub mod accessibility;
pub mod firewall;
pub mod security;

// Universal security integration for any primal with security capabilities
pub mod universal_security_integration;

// Test-related modules
#[cfg(test)]
pub mod test_impls;
#[cfg(test)]
pub mod test_types;

// Re-export universal security integration only - no deprecated APIs
pub use universal_security_integration::UniversalSecurityIntegration;

// Re-export security types
pub use security::*;
