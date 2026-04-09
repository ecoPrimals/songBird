// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Type definitions for Unix socket JSON-RPC APIs
//!
//! v3.19.1: Modern idiomatic Rust types for biomeOS integration
//! v3.20.0: Service registry types for primal discovery

mod capabilities;
mod genetic_tunnel;
mod p2p_discovery;
mod service_registry;
mod time;

pub use capabilities::{AnnounceCapabilitiesRequest, AnnounceCapabilitiesResponse};
pub use genetic_tunnel::{CreateGeneticTunnelRequest, CreateGeneticTunnelResponse, GeneticProof};
pub use p2p_discovery::{DiscoverByFamilyRequest, DiscoverByFamilyResponse, DiscoveredNode};
pub use service_registry::{
    DiscoverByCapabilityRequest, DiscoverByCapabilityResponse, GetServiceHealthRequest,
    GetServiceHealthResponse, HealthCheckRequest, HealthCheckResponse, HealthStatus,
    PrimalEndpoint, RegisterServiceRequest, RegisterServiceResponse,
};
pub use time::system_time_to_iso8601;

#[cfg(test)]
mod tests;
