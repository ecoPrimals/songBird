//! # Songbird Universal Primals
//!
//! Universal coordination system for seamless integration with all Primal ecosystem
//! components, providing automatic discovery, capability matching, and protocol adaptation.
//!
//! ## Features
//!
//! - **Universal Primal Integration**: Works with any Primal in the ecosystem
//! - **Automatic Discovery**: Discover and connect to available Primals
//! - **Capability Matching**: Match services to Primal capabilities
//! - **Protocol Adaptation**: Adapt between different Primal protocols
//! - **BearDog Integration**: Enhanced security through BearDog coordination
//! - **Dynamic Composition**: Runtime composition of Primal services
//! - **Load Balancing**: Distribute load across multiple Primals
//! - **Failover Support**: Automatic failover between Primals
//!
//! ## Architecture
//!
//! The universal-primals crate provides coordination with:
//!
//! - **BearDog**: Security and authentication services
//! - **NestGate**: Network gateway and routing services
//! - **Toadstool**: Container orchestration and management
//! - **Squirrel**: Data processing and analytics services
//! - **Custom Primals**: User-defined Primal implementations
//!
//! ## Usage
//!
//! ```rust,no_run
//! use songbird_universal_primals::{
//!     registry::UniversalPrimalRegistry,
//!     discovery::PrimalDiscovery,
//!     beardog::BearDogPrimal,
//!     config::UniversalPrimalConfig,
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize universal primal registry
//!     let registry = UniversalPrimalRegistry::new();
//!     
//!     // Create BearDog primal
//!     let beardog = BearDogPrimal::new();
//!     
//!     // Create configuration
//!     let config = UniversalPrimalConfig::default();
//!     
//!     println!("Songbird Universal Primals initialized");
//!     Ok(())
//! }
//! ```

#![warn(rust_2018_idioms)]
#![warn(missing_docs)]

/// BearDog primal integration module
///
/// This module provides configuration and integration capabilities for BearDog,
/// which handles security, authentication, and threat detection services.
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

#[allow(ambiguous_glob_reexports)]
pub use beardog::*;
#[allow(ambiguous_glob_reexports)]
pub use config::*;
#[allow(ambiguous_glob_reexports)]
pub use discovery::*;
pub use errors::*;
pub use nestgate::*;
#[allow(ambiguous_glob_reexports)]
pub use registry::*;
#[allow(ambiguous_glob_reexports)]
pub use router::*;
pub use squirrel::*;
pub use toadstool::*;
#[allow(ambiguous_glob_reexports)]
pub use traits::*;
pub use types::*;
