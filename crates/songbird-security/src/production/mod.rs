//! Production Security Implementations
//!
//! This module contains production-ready security implementations
//! that replace all mock and placeholder implementations.

pub mod real_security_provider;
pub mod production_security_adapter;

pub use real_security_provider::{ProductionSecurityProvider,
    SecurityConfig,
    SecuritySession,
    UserRecord,
    LoginAttempts};
pub use production_security_adapter::{ProductionSecurityAdapter,
    ProductionSecurityAdapterFactory,
    replace_mock_security};
