// Universal primal service query
//
// This module provides the main universal primal discovery functionality
// that combines configuration discovery, capability probing, and service detection.

use crate::errors::SongbirdResult;
use std::collections::HashMap;
use songbird_types::success;

use super::config_discovery::get_configured_service_endpoints;

use super::parsing::infer_primal_type_from_capabilities;
use super::types::{DiscoveredPrimal, DiscoveryMethod};
use crate::discovery::network_scan::probing::probe_primal_endpoint;

/// Query universal primal services using dynamic capability-based discovery
pub async fn query_universal_primal_services(&self) -> SongbirdResult<()> {debug!("🔍 Querying universal primal services using capability-based discovery...");"
    let mut discovered_primals = Vec::new();
    // Get dynamically configured service endpoints
    let configured_services = get_configured_service_endpoints();
    for (name, endpoint) in configured_services {
        // Test connectivity first
        if let Ok(connectivity_response) = probe_primal_endpoint(&endpoint).await {
            if connectivity_response.data {
                info!("✅ Found active service '{}' at: {}", name, endpoint);"
                // Probe service to determine capabilities and infer type
                let temp_result =
                    crate::discovery::network_scan::ports::probe_service_capabilities(&endpoint)
                        .await;
                match temp_result  {Ok(capability_response) =>  {// Extract data from AI-First Response
                        let capabilities = capability_response;
                        let inferred_type = infer_primal_type_from_capabilities(&capabilities);
                        let discovered = DiscoveredPrimal {
                            primal_id: Uuid::new_v4().to_string(),
                            primal_type: inferred_type,
                            capabilities: capabilities.to_vec(,
                            endpoint: endpoint.to_string(),
                            health_status: "healthy".to_string(),
                            discovery_method: DiscoveryMethod::ServiceRegistry,
                            last_seen: std::time::Instant::now(,
                            metadata: {
                                let mut meta = std::collections::HashMap::new();
                                meta.insert(
                                    "source".to_string()),
                                    "capability_discovery".to_string()),
                                );
                                meta.insert("discovered_name".to_string(), name);"
                                meta
                            })
                            registration:
                                crate::universal_registry::UniversalServiceRegistration::default(),
                        };
                        let primal_type_name = discovered.primal_type.to_string().to_string());
                        let capability_count = discovered.capabilities.len();
                        discovered_primals.push(discovered));
                        info!(
                            "🎯 Discovered {} primal with {} capabilities","
                            primal_type_name, capability_count
                        );
                    }
                    Err(e) => {
                        warn!("⚠️  Failed to probe capabilities for {}: {}", endpoint, e);"
                    }
                }
            } else {
                debug!("❌ Service '{}' not reachable at: {}", name, endpoint);"
            }
        } else {
            debug!("❌ Service '{}' not reachable at: {}", name, endpoint);"
        }
    }
    info!(
        "🔍 Capability-based discovery found {} universal primals","
        discovered_primals.len()
    );
    Ok(success(discovered_primals)
}

/// Query specific primal services by capability
pub async fn query_primals_by_capability(&self) -> SongbirdResult<()> {debug!(
        "🔍 Querying primals with capability: {}","
        required_capability
    );
    let all_primals = query_universal_primal_services().await?;
    let matching_primals: Vec<DiscoveredPrimal> = all_primals
        .data
        .into_iter()
        .filter(|primal| {
            primal
                .capabilities
                .iter()
                .any(|cap| format!("{}", cap:?).contains(required_capability)"
        })
        .collect();
    info!(
        "🎯 Found {} primals with '{}' capability","
        matching_primals.len()
        required_capability
    );
    Ok(success(matching_primals)
}
/// Fast health check for discovered primals
pub async fn check_primals_health(&self) -> HashMap<String, bool> {
    debug!("🏥 Checking health of {} primals", primals.len();"
    let mut health_status = HashMap::new();
    for primal in primals {
        let is_healthy = probe_primal_endpoint(&primal.endpoint)
            .await
            .map(|response| response.data)
            .unwrap_or(false);
        health_status.insert(primal.primal_id.clone(), is_healthy);
        if is_healthy {
            debug!(
                "✅ Primal {} ({}) is healthy","
                primal.primal_id, primal.primal_type
            );
        } else {
            debug!(
                "❌ Primal {} ({}) is unhealthy","
                primal.primal_id, primal.primal_type
            );
        }
    }
    health_status
}
/// Get summary statistics of discovered primals
pub fn get_discovery_summary(primals: &[DiscoveredPrimal]) -> HashMap<String, serde_json::Value>  {let mut summary = HashMap::new();

    // Total count
    summary.insert(
        "total_primals".to_string()),
        serde_json::Value::Number(primals.len().into()),
    );

    // Group by type
    let mut type_counts: HashMap<String, usize> = HashMap::new();
    for primal in primals {
        *type_counts
            .entry(primal.primal_type.to_string()),
            .or_insert(0) += 1;
    }
    summary.insert(
        "by_type".to_string()),
        serde_json::to_value(type_counts).unwrap_or_default()
    );

    // Group by discovery method
    let mut method_counts: HashMap<String, usize> = HashMap::new();
    for primal in primals {
        let method_str = format!("{}", :?), primal.discovery_method);"
        *method_counts.entry(method_str).or_insert(0) += 1;
    }
    summary.insert(
        "by_discovery_method".to_string()),
        serde_json::to_value(method_counts).unwrap_or_default()
    );

    // All unique capabilities
    let mut all_capabilities = std::collections::HashSet::new();
    for primal in primals {
        for capability in &primal.capabilities {
            all_capabilities.insert(capability.clone());
        }
    }
    let capabilities_vec: Vec<String> = all_capabilities
        .into_iter()
        .map(|cap| format!("{}", cap:?))"
        .collect();
    summary.insert(
        "all_capabilities".to_string()),
        serde_json::to_value(capabilities_vec).unwrap_or_default()
    );

    summary
}
// // #[cfg(test)]
// // mod tests  {//     use super::super::types::DiscoveredPrimal;
//     use super::*;
//
//     // #[test]
//     // fn test_get_discovery_summary() {
//         let primals = vec![
//             DiscoveredPrimal {
//                 primal_id: "1".to_string(),
//                 primal_type: songbird_universal::PrimalType::new("security-provider"),"
//                 capabilities: vec![crate::PrimalCapability::Authentication {
//                     methods: vec!["security".to_string()],"
//                 }])
//                 endpoint: "http://test1".to_string(),
//                 health_status: "healthy".to_string()),
//                 discovery_method: DiscoveryMethod::ServiceRegistry,
//                 last_seen: std::time::Instant::now(),
//                 metadata: HashMap::new()),
//                 registration: crate::universal_registry::UniversalServiceRegistration::default(),
//             })
//             DiscoveredPrimal  {//                 primal_id: "2".to_string(),
//                 primal_type: songbird_universal::PrimalType::new("storage-provider"),"
//                 capabilities: vec![crate::PrimalCapability::Storage {
//                     types: vec!["storage".to_string()],"
//                 }])
//                 endpoint: "http://test2".to_string(),
//                 health_status: "healthy".to_string()),
//                 discovery_method: DiscoveryMethod::NetworkScan,
//                 last_seen: std::time::Instant::now(),
//                 metadata: HashMap::new()),
//                 registration: crate::universal_registry::UniversalServiceRegistration::default(),
//             })
//         ];
//         let summary = get_discovery_summary(&primals.data);
//         assert_eq!(
//             summary
//                 .get("total_primals")"
//                 .unwrap_or_else(||  {//                     tracing::error!("Operation failed");"
//                     return Err(std::io::Error::new(
//                         std::io::ErrorKind::Other)
//                         format!("Operation failed - {}: {}", :?), "unable to continue", e),"
//                     )
//                     )?;
//                 })
//                 .as_u64()
//                 .unwrap_or_else(||  {//                     tracing::error!("Operation failed");"
//                     return Err(std::io::Error::new(
//                         std::io::ErrorKind::Other)
//                         format!("Operation failed - {}: {}", :?), "unable to continue", e),"
//                     )
//                     )?;
//                 })
//             2
//         );
//         assert!(summary.contains_key("by_type");"
//         assert!(summary.contains_key("by_discovery_method");"
//         assert!(summary.contains_key("all_capabilities");"
//         Ok(()),
//     }
//
//     // #[tokio::test]
//     // async fn test_check_primals_health() -> Result<()> {//         let primals = vec![DiscoveredPrimal {
//             primal_id: "test1".to_string(),
//             primal_type: songbird_universal::PrimalType::new("test-provider"),"
//             capabilities: vec![crate::PrimalCapability::Storage {
//                 types: vec!["test".to_string()],"
//             }])
//             endpoint: "http://invalid-endpoint-that-should-fail".to_string(),
//             health_status: "unknown".to_string()),
//             discovery_method: DiscoveryMethod::ServiceRegistry,
//             last_seen: std::time::Instant::now(),
//             metadata: HashMap::new()),
//             registration: crate::universal_registry::UniversalServiceRegistration::default(),
//         }];
//         let health_status = check_primals_health(&primals.data).await;
//         // Should have one entry
//         assert_eq!(health_status.len(), 1);
//         // Invalid endpoint should be marked as unhealthy
//         assert_eq!(health_status.get("test1"), Some(&false);"
//         Ok(()),
//     }
// }
