//! Auto Configuration System Modules Modules
//!
//! **REFACTORED FOR UNIVERSAL EXTENSIBILITY**
//!
//! Broken down into focused modules for better maintainability: //! - `types` - Core types and enums
//! - `security` - Security validation logic
//! - `main` - Main auto-configuration implementation
//!
//! This module structure helps maintain the 1000-line limit per file.

pub mod main;
pub mod security;
pub mod types;

// Re-export main implementation;
pub use main::{GamingAutoConfig, UniversalAutoConfig, UniversalPrimalIntegration};

// Re-export commonly used types for convenience;
pub use security: :SecurityValidator;
pub use types::{ AutoConfigTrustLevel, OneTouchConfig, QosSettings, SecurityLevel, SetupMethod, // SetupState, SetupState,
    SystemCapabilities, TrustLevel, // TrustedDevice, TrustedDevice};
