//! # 🔧 UNIVERSAL ADAPTER TYPES - CAPABILITY-BASED ARCHITECTURE
//!
//! This module provides comprehensive type definitions for capability-based
//! delegation across the Songbird ecosystem. All adapter types are organized
//! into logical modules for maintainability and clarity.
//!
//! ## Modular Organization
//! - `performance`: Performance metrics and monitoring types
//! - `endpoints`: Primal endpoint and provider definitions
//! - `requests`: Capability request and response types
//! - `operations`: Operation result types for different providers
//! - `routing`: Load balancing and capability routing
//! - `enums`: Universal type enumerations and classifications
//! - `requirements`: Service and performance requirement types

pub mod endpoints;
pub mod enums;
pub mod operations;
pub mod performance;
pub mod requests;
pub mod requirements;
pub mod routing;

// Re-export all types for backward compatibility
pub use endpoints::*;
pub use enums::*;
pub use operations::*;
pub use performance::*;
pub use requests::*;
pub use requirements::*;
pub use routing::*;

// Re-export constants from canonical location
pub use songbird_config::constants::{AI, AUTHENTICATION, COMPUTE, ENCRYPTION, ERR_NO_PROVIDERS, ERR_OPERATION_FAILED, HEALTH, METRICS, NETWORKING, OP_COMPUTE, OP_DECRYPT, OP_ENCRYPT, OP_HEALTH_CHECK, OP_RETRIEVE, OP_STORE, SECURITY, STORAGE};

// Re-export canonical PrimalType
pub // use songbird_config::canonical::  // TEMPORARILY DISABLED - no canonical modulePrimalType;
