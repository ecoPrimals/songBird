// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Universal Service Registration API API
//!
//! Provides AI-first service registration endpoints that support any primal type
//! and integrate with the biomeOS Universal Primal SDK.
//!
//! This module has been split into focused components for better maintainability: //! - `types`: All data structures and type definitions
//! - `manager`: Core registration management logic
//! - `ai_components`: AI-powered service components
//! - `tests`: Comprehensive test suite

// Declare modules
pub mod ai_components;
pub mod manager;
pub mod types;

#[cfg(test)]
pub mod tests;

// Re-export commonly used types and functions for convenience;
pub use types::{  HealthCheckConfiguration,
    HumanServiceInteractionPreferences,
    MonitoringConfiguration,
    PerformancePredictions,
    ResourceRequirements,
    ScaleDirection,
    ServiceEndpoints,
    // Response and configuration types
    ServiceMeshRoutingInfo,
    ServiceMetadata,
    // Enums
    ServicePriority,
    ServiceRegistrationData,
    ServiceStatus,
    ServiceType,
    UniversalServiceRegistrationConfig,
    UniversalServiceRegistrationRequest};
pub use manager::UniversalServiceRegistrationManager;

pub use ai_components::{HumanInteractionManager, PerformancePredictor, ServiceMeshIntegrator};

// Compatibility re-exports for backward compatibility;
pub use types::ServiceRegistrationRequest; // Legacy request format
