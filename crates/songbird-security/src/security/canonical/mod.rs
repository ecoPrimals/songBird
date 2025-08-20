//! # Canonical Security Patterns
//!
//! **🎯 CANONICAL UNIFICATION MODULE**
//!
//! This module contains the **canonical implementations** of all security
//! patterns in the Songbird ecosystem, providing a **single source of truth**
//! for authentication, authorization, and security operations.

pub mod authentication;

// Re-export canonical types for easy access
pub use authentication::{AuthenticationCapabilities, AuthenticationContext, AuthenticationMethod, AuthenticationRequest, CanonicalAuthenticationAdapter, CanonicalAuthenticationProvider, HealthStatus, ProviderHealth, ProviderMetadata, credentials_to_canonical_request, default_authentication_capabilities, default_provider_metadata};
