//! Songbird Orchestrator Application
//!
//! This is the main orchestrator application that coordinates all Songbird services
//! and provides the primary interface for system management.
//!
//! ## Features
//!
//! - **Service Orchestration**: Coordinate multiple services and primals
//! - **CLI Interface**: Command-line interface for system management
//! - **Web Interface**: Web-based management dashboard
//! - **Health Monitoring**: System-wide health monitoring and alerting
//! - **Integration Management**: Manage integrations with external systems
//! - **Server Management**: HTTP server for API and web interface
//!
//! ## Architecture
//!
//! The orchestrator application consists of:
//!
//! 1. **App Core**: Main application logic and coordination
//! 2. **CLI Interface**: Command-line interface for operations
//! 3. **Integration Manager**: Manages external integrations
//! 4. **Server Manager**: HTTP server for API and web interface
//!
//! ## Usage
//!
//! ```rust,no_run
//! use songbird_orchestrator::SongbirdOrchestrator;
//! use songbird_config::SongbirdConfig;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = SongbirdConfig::from_env()?;
//!     let orchestrator = SongbirdOrchestrator::new(config).await?;
//!
//!     // Start the orchestrator
//!     orchestrator.start().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Service Coordination
//!
//! The orchestrator coordinates:
//!
//! - Songbird core services
//! - Universal primals
//! - External integrations
//! - Health monitoring
//! - Performance optimization
//!
//! ## Management Interface
//!
//! Multiple management interfaces:
//!
//! - Command-line interface
//! - Web dashboard
//! - REST API
//! - Health check endpoints
//! - Metrics endpoints

#![allow(dead_code)]

pub mod app;
pub mod cli;
pub mod core; // Consolidated core functionality
              // Temporarily disabled pending syntax fixes
              // pub mod integration;
pub mod server;

// Re-export main orchestrator
pub use app::SongbirdOrchestrator;

// Re-export all functionality from crates (consolidated from songbird-lib)
// pub use songbird_cli as cli_crate;
pub use songbird_config as config;
pub use songbird_discovery as discovery;
// pub use songbird_security_errors as errors;
pub use songbird_observability as observability;
pub use songbird_registry as registry;
// pub use songbird_security_errors as security;
// pub use songbird_universal_primals as primals;

// Re-export commonly used types
pub use songbird_config::SongbirdConfig;
pub use songbird_types::{SongbirdError, SongbirdResult};

// Re-export key types that are commonly used
// pub use songbird_universal_primals::{PrimalCapability, PrimalProvider};
pub use songbird_universal::PrimalType;
