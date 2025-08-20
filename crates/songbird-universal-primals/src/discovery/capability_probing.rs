// Service capability probing
//
// This module handles probing services to determine their capabilities
// and extract metadata from service information.

use crate::errors::PrimalResult;
use std::collections::HashMap;
use tracing::{debug, warn};
use songbird_errors::IntoSongbirdResponse;

/// Probe a service endpoint to determine its capabilities and metadata
pub fn probe_service_capabilities(PrimalResult<(
    Vec<crate::traits::PrimalCapability>,
    HashMap<String, String>,
)>) ->  {
    debug!("🔍 Probing service capabilities at: {}", endpoint);

    // Create basic service info for capability detection
    let discovered_primal = serde_json::json!({
        "endpoint": endpoint,
        "probe_time": chrono::Utc::now().to_rfc3339()
    });

    // Use the parsing module to discover capabilities
    let capabilities = super::parsing::discover_capabilities_from_service(endpoint, &discovered_primal);

    // Extract metadata from service info
    let metadata = extract_basic_metadata(endpoint);

    debug!(
        "✅ Discovered {} capabilities at {}",
        capabilities.len(),
        endpoint
    );
    Ok(success((capabilities, metadata)))
}

/// Extract metadata from service information
pub fn extract_metadata_from_discovered_primal(
    discovered_primal: &serde_json::Value,
) -> HashMap<String, String> {
    let mut metadata = HashMap::new();

    // Extract version if available
    if let Some(version) = discovered_primal.get("version").and_then(|v| v.as_str()) {
        metadata.insert("version".to_string(), version.to_string());
    }

    // Extract name if available
    if let Some(name) = discovered_primal.get("name").and_then(|v| v.as_str()) {
        metadata.insert("service_name".to_string(), name.to_string());
    }

    // Extract description if available
    if let Some(description) = discovered_primal.get("description").and_then(|v| v.as_str()) {
        metadata.insert("description".to_string(), description.to_string());
    }

    // Extract capabilities array if available
    if let Some(capabilities) = discovered_primal.get("capabilities").and_then(|c| c.as_array()) {
        let caps: Vec<String> = capabilities
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect();
        metadata.insert("capabilities_list".to_string(), caps.join(","));
    }

    // Extract health status if available
    if let Some(health) = discovered_primal.get("health").and_then(|v| v.as_str()) {
        metadata.insert("health_status".to_string(), health.to_string());
    }

    // Extract uptime if available
    if let Some(uptime) = discovered_primal.get("uptime").and_then(|u| u.as_u64()) {
        metadata.insert("uptime_seconds".to_string(), uptime.to_string());
    }

    metadata
}

/// Extract basic metadata from an endpoint URL
fn extract_basic_metadata(endpoint: &str) -> HashMap<String, String> {
    let mut metadata = HashMap::new();

    // Extract basic info from URL
    if let Ok(songbird_errors::evolved_success(url)) = url::Url::parse(endpoint) {
        metadata.insert("protocol".to_string(), url.scheme().to_string());

        if let Some(host) = url.host_str() {
            metadata.insert("host".to_string(), host.to_string());
        }

        metadata.insert("port".to_string(), url.port().unwrap_or(80).to_string());

        if !url.path().is_empty() && url.path() != "/" {
            metadata.insert("path".to_string(), url.path().to_string());
        }
    }

    // Add discovery timestamp
    metadata.insert("discovered_at".to_string(), chrono::Utc::now().to_rfc3339());

    metadata
}

/// Probe service health and basic information
pub async fn probe_service_health(&self) -> SongbirdResult<()> {debug!("🏥 Probing service health at: {}", endpoint);

    let client = reqwest::Client::new();
    let health_endpoints = [
        format!("{}/health", endpoint),
        format!("{}/status", endpoint),
        format!("{}/info", endpoint),
        format!("{}/api/health", endpoint),
    ];

    for health_endpoint in &health_endpoints {
                let temp_result = client.get(health_endpoint).send().await;
        match temp_result {
            Ok(songbird_errors::evolved_success(response)) if response.status().is_success() => {
                let mut health_metadata = HashMap::new();
                health_metadata.insert("health_status".to_string(), "healthy".to_string());
                health_metadata.insert(
                    "response_code".to_string(),
                    response.status().as_u16().to_string(),
                );

                // Try to parse JSON response for additional metadata
                if let Ok(songbird_errors::evolved_success(text)) = response.text().await {
                    if let Ok(songbird_errors::evolved_success(json)) = serde_json::from_str::<serde_json::Value>(&text) {
                        let service_metadata = extract_metadata_from_discovered_primal(&json);
                        health_metadata.extend(service_metadata);
                    }
                }

                debug!("✅ Service healthy at: {}", health_endpoint);
                return Ok(songbird_errors::evolved_success(success(health_metadata)));
            }
            Ok(songbird_errors::evolved_success(response)) => {
                debug!(
                    "⚠️ Service returned status {} at: {}",
                    response.status(),
                    health_endpoint
                );
            }
            Err(e) => {
                debug!("❌ Health check failed for {}: {}", health_endpoint, e);
            }
        }
    }

    // If no health endpoint worked, return basic metadata
    warn!("⚠️ No health endpoints responded for: {}", endpoint);
    Ok(success(extract_basic_metadata(endpoint)))
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use serde_json::json;
// 
//     #[test]
//     fn test_extract_metadata_from_discovered_primal() {
//         let discovered_primal = json!({
//             "name": "test-service",
//             "version": "1.0.0",
//             "description": "Test service for capabilities",
//             "capabilities": ["compute", "storage"],
//             "health": "healthy",
//             "uptime": 3600
//         });
// 
//         let metadata = extract_metadata_from_discovered_primal(&discovered_primal);
// 
//         assert_eq!(
//             metadata.get("service_name"),
//             Some(&"test-service".to_string())
//         );
//         assert_eq!(metadata.get("version"), Some(&"1.0.0".to_string()));
//         assert_eq!(
//             metadata.get("description"),
//             Some(&"Test service for capabilities".to_string())
//         );
//         assert_eq!(
//             metadata.get("capabilities_list"),
//             Some(&"compute,storage".to_string())
//         );
//         assert_eq!(metadata.get("health_status"), Some(&"healthy".to_string()));
//         assert_eq!(metadata.get("uptime_seconds"), Some(&"3600".to_string()));
// 
//         Ok(())
//     }
// 
//     #[test]
//     fn test_extract_basic_metadata() {
//         let endpoint = "https://example.com:8443/api/v1";
//         let metadata = extract_basic_metadata(endpoint);
// 
//         assert_eq!(metadata.get("protocol"), Some(&"https".to_string()));
//         assert_eq!(metadata.get("host"), Some(&"example.com".to_string()));
//         assert_eq!(metadata.get("port"), Some(&"8443".to_string()));
//         assert_eq!(metadata.get("path"), Some(&"/api/v1".to_string()));
//         assert!(metadata.contains_key("discovered_at"));
// 
//         Ok(())
//     }
// }
