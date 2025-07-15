//! # Songbird Configuration
//!
//! Flexible configuration management system providing environment-aware configuration,
//! validation, and zero-touch deployment capabilities.
//!
//! ## Features
//!
//! - **Environment-Aware**: Configuration that adapts to different environments
//! - **Validation**: Comprehensive configuration validation and error reporting
//! - **Zero-Touch Deployment**: Automatic configuration discovery and setup
//! - **Hardcoded Value Elimination**: Dynamic configuration without hardcoded values
//! - **Hot Reload**: Live configuration updates without service restarts
//! - **Multi-Format Support**: TOML, YAML, JSON, and environment variables
//! - **Hierarchical Configuration**: Layered configuration with inheritance
//! - **Secret Management**: Secure handling of sensitive configuration data
//!
//! ## Architecture
//!
//! The configuration crate is organized into focused modules:
//!
//! - `config`: Core configuration management and validation
//! - `zero_touch`: Zero-touch deployment and auto-configuration
//!
//! ## Usage
//!
//! ```rust,no_run
//! use songbird_config::{
//!     config::{SongbirdConfig, ConfigBuilder},
//!     zero_touch::{ZeroTouchDeployment, DeploymentConfig},
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration from multiple sources
//!     let config = ConfigBuilder::new()
//!         .add_file("config.toml")
//!         .add_env_vars()
//!         .build()?;
//!     
//!     // Initialize zero-touch deployment
//!     let deployment_config = DeploymentConfig::default();
//!     let zero_touch = ZeroTouchDeployment::new(deployment_config);
//!     
//!     // Auto-configure the system
//!     zero_touch.auto_configure().await?;
//!     
//!     println!("Songbird configuration loaded and validated");
//!     Ok(())
//! }
//! ```
//!
//! ## Configuration Sources
//!
//! The configuration system supports multiple sources in order of precedence:
//!
//! 1. **Command Line Arguments**: Highest priority
//! 2. **Environment Variables**: Override file-based configuration
//! 3. **Configuration Files**: TOML, YAML, JSON formats
//! 4. **Default Values**: Fallback defaults for all settings
//!
//! ## Zero-Touch Deployment
//!
//! Automatic configuration discovery and setup:
//!
//! - **Environment Detection**: Automatically detect deployment environment
//! - **Service Discovery**: Discover and configure dependent services
//! - **Network Configuration**: Auto-configure network settings
//! - **Security Setup**: Automatic security policy application
//! - **Performance Tuning**: Environment-specific performance optimization
//!
//! ## Validation
//!
//! Comprehensive configuration validation including:
//!
//! - **Schema Validation**: Type and structure validation
//! - **Range Validation**: Numeric and string range constraints
//! - **Dependency Validation**: Cross-configuration dependency checks
//! - **Environment Validation**: Environment-specific constraint validation
//! - **Security Validation**: Security policy compliance checks
//!
//! ## Error Handling
//!
//! All configuration operations return `Result<T, SongbirdError>` with detailed
//! error information including:
//!
//! - Configuration path and line number for file errors
//! - Environment variable names for environment errors
//! - Validation failure details with suggested corrections
//! - Recovery suggestions for common configuration issues

#![allow(clippy::multiple_crate_versions)]

pub mod config;
pub mod zero_touch;

pub use config::*;
pub use zero_touch::*;
