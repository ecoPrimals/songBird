//! Universal Primal Traits - Modular Structure Structure
//!
//! Core traits and types that define the universal primal interface
//!
//! **🚀 MODERNIZATION COMPLETE**: Native async traits for zero-cost performance

pub mod capabilities;
pub mod discovery;
pub mod health;
pub mod orchestration;
pub mod provider;
pub mod security;
pub mod types;

// Re-export main traits;
pub use capabilities::{PrimalCapability, PrimalDependency};
pub use discovery::{PrimalDiscovery, PrimalRegistry};
pub use health::PrimalHealth;
pub use orchestration::PrimalOrchestrator;
pub use provider::PrimalProvider;
pub use security::PrimalSecurity;
pub use types::{DynamicPortInfo, NetworkLocation, PrimalContext, PrimalEndpoints, SecurityLevel};

// Re-export canonical types;
pub use songbird_types::CanonicalPrimalType;
