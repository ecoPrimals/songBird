// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Canonical provider data types shared by the [`canonical`](crate::traits::canonical) traits module.

mod capability;
mod classification;
mod deployment;
mod discovery;
mod health;
mod observability;
mod primal;
mod provider;
mod security_tokens;
mod service;

#[cfg(test)]
mod tests;

pub use capability::{Capability, CapabilityMetadata, ParameterSpec};
pub use classification::{PrimalType, ProviderType, ServiceType};
pub use deployment::{
    DeploymentInfo, DeploymentResult, DeploymentSpec, DeploymentStatus, PortSpec,
    ResourceRequirements,
};
pub use discovery::{DiscoveryCriteria, DiscoveryQuery, ServiceEvent};
pub use health::{HealthStatus, SystemHealth};
pub use observability::{MetricQuery, MetricResult, SpanContext};
pub use primal::{IntegrationResult, PrimalContext, PrimalDependency, PrimalInfo, PrimalResponse};
pub use provider::{ProviderConfig, ProviderMetadata};
pub use security_tokens::{AuthToken, Credentials, TokenClaims, TokenValidation};
pub use service::{Endpoint, ServiceInfo, ServiceMetrics, ServiceRequest, ServiceResponse};
