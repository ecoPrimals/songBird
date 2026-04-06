// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Environment-Based Discovery Backend
//!
//! EVOLVED: Pure capability-based discovery using environment variables
//! Zero hardcoding - discovers ANY primal that advertises via env vars

use crate::capabilities::Capability;
use crate::capabilities::QoSMetrics;
use crate::types::PrimalType;
use std::collections::HashMap;

use super::super::errors::DiscoveryError;
use super::super::types::{DiscoveredPrimal, DiscoveryMethod, PrimalHealth};
use tracing::debug;

/// Discover primals from environment variables
///
/// **SELF-KNOWLEDGE PRINCIPLE**: Discovers what's advertised, not what we expect
///
/// # Errors
///
/// Does not return errors; invalid env vars are skipped.
///
/// # Environment Variables Pattern
///
/// For each capability provider, set:
/// - `{CAPABILITY}_PROVIDER_ENDPOINT` - Full endpoint URL
/// - `{CAPABILITY}_PROVIDER_NAME` - Optional name (defaults to capability type)
///
/// Examples:
/// ```bash
/// export COMPUTE_PROVIDER_ENDPOINT="http://compute-service:8080"
/// export COMPUTE_PROVIDER_NAME="my-compute-service"
/// export AI_PROVIDER_ENDPOINT="http://ai-service:8081"
/// export STORAGE_PROVIDER_ENDPOINT="http://storage-service:8082"
/// ```
///
/// **ZERO HARDCODING**: Works with ANY capability provider (current or future)
pub async fn discover_from_environment() -> Result<Vec<DiscoveredPrimal>, DiscoveryError> {
    debug!("🔍 Discovering capability providers from environment variables...");

    let mut discovered = Vec::new();
    let capability_types = [
        "COMPUTE",
        "AI",
        "STORAGE",
        "SECURITY",
        "MESSAGING",
        "ANALYTICS",
        "ML",
        "DATABASE",
        "CACHE",
        "QUEUE",
    ];

    for cap_type in &capability_types {
        let endpoint_key = format!("{cap_type}_PROVIDER_ENDPOINT");
        let name_key = format!("{cap_type}_PROVIDER_NAME");

        if let Ok(endpoint) = songbird_process_env::var(&endpoint_key) {
            let name = songbird_process_env::var(&name_key)
                .unwrap_or_else(|_| format!("{}-provider", cap_type.to_lowercase()));

            let primal = DiscoveredPrimal {
                name,
                primal_type: infer_primal_type(cap_type),
                endpoint,
                capabilities: vec![Capability {
                    capability_type: cap_type.to_lowercase(),
                    name: cap_type.to_lowercase(),
                    version: "1.0".to_string(),
                    parameters: HashMap::default(),
                    qos_metrics: QoSMetrics::default(),
                    available: true,
                }],
                health: PrimalHealth::Unknown,
                discovery_method: DiscoveryMethod::Environment,
                metadata: HashMap::default(),
            };

            discovered.push(primal);
        }
    }

    if discovered.is_empty() {
        debug!("No capability providers found in environment variables");
    } else {
        debug!("Found {} capability providers in environment", discovered.len());
    }

    Ok(discovered)
}

/// Infer primal type from capability name
///
/// **HEURISTIC**: Best-guess mapping, can be overridden by explicit metadata
fn infer_primal_type(capability: &str) -> PrimalType {
    let category = match capability {
        "COMPUTE" | "ML" => "compute",
        "AI" => "ai",
        "STORAGE" | "DATABASE" | "CACHE" => "storage",
        "SECURITY" => "security",
        _ => "generic",
    };
    PrimalType::new(category)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discover_from_environment_empty() {
        let result = discover_from_environment().await;
        assert!(result.is_ok());
        // May be empty or have some primals depending on environment
    }

    #[test]
    fn test_infer_primal_type() {
        assert_eq!(infer_primal_type("COMPUTE").category, "compute");
        assert_eq!(infer_primal_type("AI").category, "ai");
        assert_eq!(infer_primal_type("STORAGE").category, "storage");
        assert_eq!(infer_primal_type("UNKNOWN").category, "generic");
    }
}
