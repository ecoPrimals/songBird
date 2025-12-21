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
//! use songbird_types::config::CanonicalSongbirdConfig;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration from environment variables
//!     let config = CanonicalSongbirdConfig::from_env()?;
//!     let mut orchestrator = SongbirdOrchestrator::new(config).await?;
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

#![allow(
    dead_code,
    unused_variables,
    clippy::missing_errors_doc,
    clippy::used_underscore_binding,
    clippy::struct_excessive_bools,
    clippy::too_many_lines,
    clippy::cast_sign_loss,
    clippy::no_effect_underscore_binding,
    clippy::unused_async
)]

pub mod access_control; // Access control & graduated information disclosure (Q1 2025)
pub mod app;
pub mod cli;
pub mod consent_management;
pub mod core; // Consolidated core functionality
pub mod error_recovery; // Error recovery & resilience (Week 3 - Dec 18, 2025)
pub mod integration;
pub mod network; // Network binding & endpoint management (Dec 20, 2025) - Zero-config intelligent binding
pub mod node_identity; // Stable node identity (Dec 20, 2025) - Multi-path transport foundation
pub mod observability; // Basic observability (Week 4 - Dec 18, 2025)
pub mod orchestrator; // MVP Integration (Week 1-5 - Dec 18, 2025)
pub mod privilege; // Secure privilege management (Dec 20, 2025) - CAP_NET_ADMIN, no sudo prompts
pub mod process_manager; // Process lifecycle & singleton enforcement (Dec 20, 2025) - Prevents split state bug
pub mod resource_management; // Resource management & fairness (Week 2 - Dec 18, 2025)
pub mod rpc; // Multi-protocol RPC (JSON-RPC, tarpc)
pub mod server;
pub mod service_registry; // Universal Port Authority (Dec 20, 2025) - Inter-primal service registration
pub mod task_lifecycle; // Task lifecycle management (Week 1 - Dec 18, 2025) // Consent management (Week 5 - Dec 18, 2025)
pub mod trust; // Trust escalation system (Dec 19, 2025) - Zero-trust progressive escalation

// Re-export main orchestrator
pub use app::SongbirdOrchestrator;

// Re-export MVP orchestrator integration
pub use orchestrator::{HealthStatus, OrchestratorConfig, SongbirdOrchestrator as MvpOrchestrator};

// Re-export all functionality from crates (consolidated from songbird-lib)
// pub use songbird_cli as cli_crate;
pub use songbird_config as config;
pub use songbird_discovery as discovery;
// pub use songbird_security_errors as errors;
// pub use songbird_observability as observability; // Commented out - using new observability module (Week 4)
pub use songbird_registry as registry;
// pub use songbird_security_errors as security;
// pub use songbird_universal_primals as primals;

// Re-export commonly used types
pub use songbird_types::config::CanonicalSongbirdConfig;
pub use songbird_types::{SongbirdError, SongbirdResult};

// Re-export key types that are commonly used
// pub use songbird_universal_primals::{PrimalCapability, PrimalProvider};
pub use songbird_universal::PrimalType;
