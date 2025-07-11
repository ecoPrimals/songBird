//! # Songbird Universal Primal System
//!
//! This crate provides a universal integration system for Songbird that automatically
//! discovers and integrates with standalone primal services through HTTP APIs.
//!
//! ## Architecture
//!
//! The universal primal system consists of several key components:
//!
//! - **Primal Registry**: Central registry for discovering and managing primal instances
//! - **Primal Providers**: Individual adapter implementations for each primal service
//! - **Communication Protocol**: Standardized request/response format for all primals
//! - **Multi-Instance Support**: Ability to manage multiple instances of the same primal type
//! - **Context-Aware Routing**: Route requests to appropriate primal instances based on user/device context
//!
//! ## Supported Primals
//!
//! - **BearDog**: Security primal (authentication, encryption, authorization)
//! - **NestGate**: Storage primal (data persistence, backup, retrieval)
//! - **Toadstool**: Compute primal (container execution, serverless functions)
//! - **Squirrel**: AI primal (model inference, natural language processing)
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use songbird_universal_primals::{UniversalPrimalRegistry, PrimalContext};
//! use std::collections::HashMap;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create registry
//! let mut registry = UniversalPrimalRegistry::new();
//!
//! // Auto-discover primals
//! let _discovered = registry.auto_discover().await?;
//!
//! // Create a request context
//! let context = PrimalContext::default();
//!
//! // The registry is now ready to handle primal requests
//! println!("Universal primal registry initialized successfully");
//! # Ok(())
//! # }
//! ```

#![deny(unsafe_code)]
#![warn(clippy::all)]
#![warn(rust_2018_idioms)]
#![warn(missing_docs)]

pub mod beardog;
pub mod config;
pub mod discovery;
pub mod errors;
pub mod nestgate;
pub mod registry;
pub mod router;
pub mod squirrel;
pub mod toadstool;
pub mod traits;
pub mod types;

// Re-export commonly used types
pub use beardog::BearDogPrimal;
pub use config::{UniversalPrimalConfig, PrimalInstanceConfig, MultiInstanceConfig};
pub use errors::{PrimalError, PrimalResult};
pub use nestgate::NestGatePrimal;
pub use registry::UniversalPrimalRegistry;
pub use squirrel::SquirrelPrimal;
pub use toadstool::ToadstoolPrimal;
pub use traits::{
    PrimalProvider, PrimalType, PrimalCapability, PrimalHealth, PrimalContext,
    DynamicPortInfo, SecurityLevel
};
pub use types::{PrimalRequest, PrimalResponse, PrimalRequestType, PrimalResponseType};

/// Initialize the universal primal system
/// 
/// This function sets up the registry and performs initial configuration
/// based on environment variables and provided configuration.
///
/// # Arguments
///
/// * `config` - Optional configuration for the primal system
///
/// # Returns
///
/// A configured `UniversalPrimalRegistry` instance
///
/// # Example
///
/// ```rust,no_run
/// use songbird_universal_primals::initialize_primal_system;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let registry = initialize_primal_system(None).await?;
/// # Ok(())
/// # }
/// ```
pub async fn initialize_primal_system(
    config: Option<UniversalPrimalConfig>
) -> PrimalResult<UniversalPrimalRegistry> {
    let config = config.unwrap_or_else(UniversalPrimalConfig::from_env);
    let mut registry = UniversalPrimalRegistry::new();
    
    // Perform auto-discovery if enabled
    if config.auto_discovery_enabled {
        registry.auto_discover().await?;
    }
    
    Ok(registry)
}

/// Create a new BearDog primal instance
/// 
/// This is a convenience function for creating BearDog security primal instances
/// with default configuration.
///
/// # Arguments
///
/// * `base_url` - Base URL for the BearDog service (unused in current implementation)
/// * `context` - User/device context for this instance
///
/// # Returns
///
/// A configured `BearDogPrimal` instance
pub async fn create_beardog_primal(
    _base_url: &str,
    context: PrimalContext
) -> PrimalResult<BearDogPrimal> {
    Ok(BearDogPrimal::with_context(context))
}

/// Create a new NestGate primal instance
/// 
/// This is a convenience function for creating NestGate storage primal instances
/// with default configuration.
///
/// # Arguments
///
/// * `base_url` - Base URL for the NestGate service (unused in current implementation)
/// * `context` - User/device context for this instance
///
/// # Returns
///
/// A configured `NestGatePrimal` instance
pub async fn create_nestgate_primal(
    _base_url: &str,
    context: PrimalContext
) -> PrimalResult<NestGatePrimal> {
    Ok(NestGatePrimal::with_context(context))
}

/// Create a new Toadstool primal instance
/// 
/// This is a convenience function for creating Toadstool compute primal instances
/// with default configuration.
///
/// # Arguments
///
/// * `base_url` - Base URL for the Toadstool service (unused in current implementation)
/// * `context` - User/device context for this instance
///
/// # Returns
///
/// A configured `ToadstoolPrimal` instance
pub async fn create_toadstool_primal(
    _base_url: &str,
    context: PrimalContext
) -> PrimalResult<ToadstoolPrimal> {
    Ok(ToadstoolPrimal::new(context))
}

/// Create a new Squirrel primal instance
/// 
/// This is a convenience function for creating Squirrel AI primal instances
/// with default configuration.
///
/// # Arguments
///
/// * `base_url` - Base URL for the Squirrel service (unused in current implementation)
/// * `context` - User/device context for this instance
///
/// # Returns
///
/// A configured `SquirrelPrimal` instance
pub async fn create_squirrel_primal(
    _base_url: &str,
    context: PrimalContext
) -> PrimalResult<SquirrelPrimal> {
    Ok(SquirrelPrimal::new(context))
} 