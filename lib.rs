//! Songbird Universal Orchestrator
//!
//! A universal orchestration platform that coordinates multiple standalone services
//! in the ecoPrimals ecosystem using toadstool and biomeOS as the OS substrate.

#![deny(unsafe_code)]
#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

// Re-export all functionality from crates
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

// Re-export commonly used types from crates
pub use songbird_errors::{Result, SongbirdError};
pub use songbird_config::SongbirdConfig;

// Re-export core functionality for easy access
pub use songbird_core::{
    orchestrator::*,
    load_balancer::*,
    scalability::*,
    robustness::*,
    benchmarks::*,
    performance_optimizer::*,
    production_benchmarks::*,
    biomeos_integration::*,
};

// Re-export network functionality
pub use songbird_network::{
    communication::*,
    network::*,
    proxy::*,
    http_server::*,
    management::*,
};

// Re-export security functionality
pub use songbird_security::{
    security::*,
    firewall::*,
    accessibility::*,
};

// Re-export observability functionality
pub use songbird_observability::{
    observability::*,
    health::*,
};

// Re-export CLI functionality
pub use songbird_cli::cli::*;

// Re-export discovery functionality
pub use songbird_discovery::*;

// Re-export federation functionality
pub use songbird_federation::*;

// Re-export registry functionality
pub use songbird_registry::*;

// Re-export universal primals functionality
pub use songbird_universal_primals::*; 