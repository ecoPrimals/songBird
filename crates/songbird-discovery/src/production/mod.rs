//! Production Discovery Implementations
//!
//! This module contains production-ready service discovery implementations
//! that replace all mock and placeholder discovery providers.

pub mod real_service_discovery;

pub use real_service_discovery::ServiceHealthStatus;
