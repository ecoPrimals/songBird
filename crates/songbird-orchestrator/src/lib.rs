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
//! ```rust
//! use songbird_orchestrator::SongbirdOrchestrator;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let orchestrator = SongbirdOrchestrator::new().await?;
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
pub mod integration;
pub mod server;

pub use app::SongbirdOrchestrator;
