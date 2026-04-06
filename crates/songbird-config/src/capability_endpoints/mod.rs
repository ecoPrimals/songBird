// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # 🍼 Capability-Based Endpoints (Zero Hardcoding)
//!
//! **PHILOSOPHY**: Request capabilities (security, storage, compute, ai), not specific providers.
//!
//! This module replaces primal-name-based endpoint configuration with capability-based
//! discovery. Services specify WHAT they need, not WHO provides it.
//!
//! ## Migration from Legacy
//!
//! ```rust,ignore
//! // ❌ OLD: Hardcoded primal names
//! let endpoint = endpoints::get_primal_endpoint("beardog");
//!
//! // ✅ NEW: Capability-based
//! let endpoint = capability_endpoints::get_capability_endpoint("security").await?;
//! ```
//!
//! ## Environment Variables
//!
//! ### Capability Endpoints (Optional - discovered if not set)
//! - `CAPABILITY_SECURITY_ENDPOINT` - Security provider endpoint
//! - `CAPABILITY_STORAGE_ENDPOINT` - Storage provider endpoint
//! - `CAPABILITY_COMPUTE_ENDPOINT` - Compute provider endpoint
//! - `CAPABILITY_AI_ENDPOINT` - AI provider endpoint
//! - `CAPABILITY_ORCHESTRATION_ENDPOINT` - Orchestration provider endpoint
//!
//! ### Discovery Configuration
//! - `SERVICE_REGISTRY_ENDPOINT` - Service registry for discovery
//! - `ENABLE_INFANT_DISCOVERY` - Enable zero-knowledge bootstrap
//! - `DISCOVERY_TIMEOUT_SECS` - Discovery timeout (default: 30)
//!
//! ## Submodules
//!
//! - `types` — capability enum and endpoint records
//! - `resolver` — cache and orchestrated resolution
//! - `remote_probes` — registry / container / DNS probing
//! - `api` — stateless convenience functions

mod api;
mod remote_probes;
mod resolver;
mod types;

pub use api::{
    clear_cache, get_all_endpoints, get_capability_endpoint, get_endpoint_typed,
    get_multiple_endpoints, has_capability,
};
pub use resolver::CapabilityEndpointResolver;
pub use types::{CapabilityEndpoint, CapabilityType, DiscoveryMethod};

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests;
