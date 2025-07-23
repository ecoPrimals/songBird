//! Songbird Universal Primals
//!
//! This crate provides universal integration patterns for connecting with
//! any primal in the ecoPrimals ecosystem through capability-based discovery.

pub mod beardog;
pub mod config;
pub mod discovery;
pub mod errors;
// Replace the large universal_registry.rs with modular structure
pub mod nestgate;
pub mod router;
pub mod squirrel;
pub mod toadstool;
pub mod traits;
pub mod types;
pub mod universal_registry;

// Re-export main APIs with specific items to avoid ambiguous re-exports
pub use beardog::*;
pub use discovery::*;
pub use errors::*;
pub use nestgate::*;
pub use squirrel::*;
pub use toadstool::*;
pub use types::*;

// Re-export universal registry types but use specific imports
pub use universal_registry::{
    MemoryServiceRegistry, ServiceCapability, ServiceHandle, ServiceInfo, ServiceMetadata,
    UniversalServiceRegistration, UniversalServiceRegistry,
};

// Re-export traits but be specific about what we need
pub use traits::{
    DynamicPortInfo, PrimalCapability, PrimalContext, PrimalDependency, PrimalHealth,
    PrimalProvider, SecurityLevel,
};

// Re-export config but be specific
pub use config::UniversalPrimalConfig;
