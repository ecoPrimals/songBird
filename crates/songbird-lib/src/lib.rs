//! Songbird Universal Orchestrator
//!
//! A universal orchestration platform that coordinates multiple standalone services
//! in the ecoPrimals ecosystem using toadstool and biomeOS as the OS substrate.

#![deny(unsafe_code)]
#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

// Re-export all functionality from crates (these are safe and don't cause conflicts)
pub use songbird_cli as cli;
pub use songbird_config as config;
pub use songbird_core as core;
pub use songbird_discovery as discovery;
pub use songbird_errors as errors;
pub use songbird_federation as federation;
pub use songbird_network as network;
pub use songbird_observability as observability;
pub use songbird_registry as registry;
pub use songbird_security as security;
pub use songbird_universal_primals as primals;

// Re-export commonly used types (confirmed to exist)
pub use songbird_config::SongbirdConfig;
pub use songbird_errors::{Result, SongbirdError};

// Re-export key types that are commonly used and confirmed to exist
pub use songbird_universal_primals::{PrimalCapability, PrimalProvider};

pub use songbird_federation::FederationManager;
pub use songbird_network::network::discovery::engine::NetworkDiscoveryEngine;
pub use songbird_universal::PrimalType;
