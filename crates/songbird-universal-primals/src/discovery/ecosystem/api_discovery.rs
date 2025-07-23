//! API-based Service Capability Discovery
//!
//! Discovers service capabilities by probing API endpoints and analyzing responses

use crate::errors::{PrimalError, PrimalResult};
use crate::traits::PrimalCapability;
use serde_json::Value;
use songbird_universal::PrimalType;
use tracing::{debug, info, warn};

/// Get capabilities for a primal through self-advertisement, not name inference
pub async fn discover_service_capabilities_via_api(
    http_client: &reqwest::Client,
    endpoint: &str,
    primal_name: &str,
) -> PrimalResult<(PrimalType, Vec<PrimalCapability>)> {
    debug!("🔍 Discovering capabilities for service at: {}", endpoint);

    // Try various standard capability discovery endpoints
    let capability_endpoints = [
        "/api/v1/capabilities",
        "/capabilities",
        "/api/capabilities",
        "/api/v1/service-info",
        "/service-info",
        "/api/v1/registration",
        "/registration",
        "/.well-known/service-capabilities",
    ];

    for cap_endpoint in &capability_endpoints {
        let full_url = format!("{endpoint}{cap_endpoint}");
        debug!("  🌐 Trying capability endpoint: {}", full_url);

        match try_discover_from_endpoint(http_client, &full_url).await {
            Ok(Some(info)) => {
                debug!("  ✅ Got capability info from: {}", cap_endpoint);

                if let Some(capabilities) = extract_capabilities_from_response(&info) {
                    let primal_type = infer_primal_type_from_capabilities(&capabilities);
                    info!(
                        "🎯 Discovered {} capabilities via API for {} [{}]",
                        capabilities.len(),
                        primal_name,
                        primal_type.as_str()
                    );
                    return Ok((primal_type, capabilities));
                }
            }
            Ok(None) => {
                debug!("  ❌ No capability info at: {}", cap_endpoint);
            }
            Err(e) => {
                debug!("  ⚠️ Failed to query {}: {}", cap_endpoint, e);
            }
        }
    }

    // If direct capability discovery fails, try behavior-based inference
    info!(
        "🧠 Direct capability discovery failed for {}, trying behavior inference",
        primal_name
    );

    let inferred_capabilities =
        infer_capabilities_from_endpoint_behavior(http_client, endpoint).await;

    if !inferred_capabilities.is_empty() {
        let primal_type = infer_primal_type_from_capabilities(&inferred_capabilities);
        info!(
            "🔮 Inferred {} capabilities from behavior for {} [{}]",
            inferred_capabilities.len(),
            primal_name,
            primal_type.as_str()
        );
        return Ok((primal_type, inferred_capabilities));
    }

    // Final fallback: use name-based inference (but still universal)
    warn!(
        "🔄 All API-based discovery failed for {}, using fallback inference",
        primal_name
    );
    let fallback_caps =
        super::capability_inference::get_default_capabilities_for_primal(primal_name);
    Ok(fallback_caps)
}

/// Try to discover capabilities from a specific endpoint
async fn try_discover_from_endpoint(
    http_client: &reqwest::Client,
    url: &str,
) -> PrimalResult<Option<Value>> {
    match http_client.get(url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<Value>().await {
                    Ok(json) => Ok(Some(json)),
                    Err(e) => {
                        debug!("Failed to parse JSON from {url}: {e}");
                        Ok(None)
                    }
                }
            } else {
                debug!("HTTP error {} from: {url}", response.status());
                Ok(None)
            }
        }
        Err(e) => {
            debug!("Request failed to {url}: {e}");
            Err(PrimalError::discovery_error(format!(
                "HTTP request failed: {e}"
            )))
        }
    }
}

/// Extract capabilities from API response
fn extract_capabilities_from_response(info: &Value) -> Option<Vec<PrimalCapability>> {
    let mut capabilities = Vec::new();

    // Try different JSON structures for capability advertisement
    if let Some(caps) = info.get("capabilities").and_then(|c| c.as_array()) {
        for cap in caps {
            if let Some(cap_obj) = string_to_capability(cap) {
                capabilities.push(cap_obj);
            }
        }
    } else if let Some(services) = info.get("services").and_then(|s| s.as_array()) {
        // Alternative structure where services are listed
        for service in services {
            if let Some(cap_obj) = service_to_capability(service) {
                capabilities.push(cap_obj);
            }
        }
    } else if let Some(features) = info.get("features").and_then(|f| f.as_array()) {
        // Another alternative structure where features are listed
        for feature in features {
            if let Some(cap_obj) = feature_to_capability(feature) {
                capabilities.push(cap_obj);
            }
        }
    }

    if capabilities.is_empty() {
        None
    } else {
        Some(capabilities)
    }
}

/// Convert string capability to PrimalCapability enum
fn string_to_capability(cap_value: &Value) -> Option<PrimalCapability> {
    if let Some(cap_str) = cap_value.as_str() {
        match cap_str.to_lowercase().as_str() {
            "auth" | "authentication" => Some(PrimalCapability::Authentication {
                methods: vec!["api".to_string()],
            }),
            "encrypt" | "encryption" => Some(PrimalCapability::Encryption {
                algorithms: vec!["aes256".to_string()],
            }),
            "storage" | "file" | "files" => Some(PrimalCapability::FileSystem {
                supports_zfs: false,
            }),
            "object" | "objects" => Some(PrimalCapability::ObjectStorage {
                backends: vec!["local".to_string()],
            }),
            "container" | "containers" | "docker" => Some(PrimalCapability::ContainerRuntime {
                orchestrators: vec!["docker".to_string()],
            }),
            "serverless" | "lambda" | "functions" => Some(PrimalCapability::ServerlessExecution {
                languages: vec!["universal".to_string()],
            }),
            "ai" | "ml" | "model" | "inference" => Some(PrimalCapability::ModelInference {
                models: vec!["llm".to_string()],
            }),
            "agent" | "agents" => Some(PrimalCapability::AgentFramework { mcp_support: false }),
            "orchestration" | "orchestrate" => Some(PrimalCapability::Orchestration {
                features: vec!["universal".to_string()],
            }),
            "discovery" | "service-discovery" => Some(PrimalCapability::ServiceDiscovery {
                protocols: vec!["http".to_string()],
            }),
            _ => {
                debug!("Unknown capability string: {}", cap_str);
                None
            }
        }
    } else if let Some(cap_obj) = cap_value.as_object() {
        // Handle capability objects with more details
        object_to_capability(cap_obj)
    } else {
        None
    }
}

/// Convert service info to capability
fn service_to_capability(service: &Value) -> Option<PrimalCapability> {
    if let Some(service_type) = service.get("type").and_then(|t| t.as_str()) {
        string_to_capability(&Value::String(service_type.to_string()))
    } else {
        None
    }
}

/// Convert feature info to capability
fn feature_to_capability(feature: &Value) -> Option<PrimalCapability> {
    if let Some(feature_name) = feature.get("name").and_then(|n| n.as_str()) {
        string_to_capability(&Value::String(feature_name.to_string()))
    } else if let Some(feature_str) = feature.as_str() {
        string_to_capability(&Value::String(feature_str.to_string()))
    } else {
        None
    }
}

/// Convert capability object to PrimalCapability enum
fn object_to_capability(obj: &serde_json::Map<String, Value>) -> Option<PrimalCapability> {
    if let Some(cap_type) = obj.get("type").and_then(|t| t.as_str()) {
        match cap_type.to_lowercase().as_str() {
            "authentication" => Some(PrimalCapability::Authentication {
                methods: obj
                    .get("methods")
                    .and_then(|m| m.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_else(|| vec!["oauth2".to_string()]),
            }),
            "encryption" => Some(PrimalCapability::Encryption {
                algorithms: obj
                    .get("algorithms")
                    .and_then(|a| a.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_else(|| vec!["aes256".to_string()]),
            }),
            "storage" => Some(PrimalCapability::FileSystem {
                supports_zfs: obj
                    .get("supports_zfs")
                    .and_then(|z| z.as_bool())
                    .unwrap_or(false),
            }),
            _ => {
                debug!("Unknown capability object type: {}", cap_type);
                None
            }
        }
    } else {
        None
    }
}

/// Infer capabilities from endpoint behavior (probe endpoints)
async fn infer_capabilities_from_endpoint_behavior(
    http_client: &reqwest::Client,
    endpoint: &str,
) -> Vec<PrimalCapability> {
    let mut capabilities = Vec::new();

    // Test for different service types by trying common endpoints
    let capability_tests = vec![
        // Security service tests
        (
            vec!["/auth", "/login", "/api/auth", "/api/v1/auth"],
            PrimalCapability::Authentication {
                methods: vec!["http".to_string()],
            },
        ),
        (
            vec!["/encrypt", "/decrypt", "/api/crypto", "/api/v1/crypto"],
            PrimalCapability::Encryption {
                algorithms: vec!["unknown".to_string()],
            },
        ),
        // Storage service tests
        (
            vec!["/files", "/storage", "/api/files", "/api/v1/storage"],
            PrimalCapability::FileSystem {
                supports_zfs: false,
            },
        ),
        (
            vec!["/objects", "/blob", "/api/objects", "/api/v1/blob"],
            PrimalCapability::ObjectStorage {
                backends: vec!["local".to_string()],
            },
        ),
        // Compute service tests
        (
            vec![
                "/containers",
                "/docker",
                "/api/containers",
                "/api/v1/containers",
            ],
            PrimalCapability::ContainerRuntime {
                orchestrators: vec!["docker".to_string()],
            },
        ),
        (
            vec![
                "/functions",
                "/lambda",
                "/api/functions",
                "/api/v1/serverless",
            ],
            PrimalCapability::ServerlessExecution {
                languages: vec!["universal".to_string()],
            },
        ),
        // AI service tests
        (
            vec!["/models", "/inference", "/ai", "/api/models"],
            PrimalCapability::ModelInference {
                models: vec!["llm".to_string()],
            },
        ),
        (
            vec!["/agents", "/api/agents", "/mcp"],
            PrimalCapability::AgentFramework { mcp_support: false },
        ),
        // Orchestration service tests
        (
            vec!["/orchestrate", "/api/orchestration", "/manage"],
            PrimalCapability::Orchestration {
                features: vec!["universal".to_string()],
            },
        ),
        (
            vec!["/discover", "/api/discovery", "/services"],
            PrimalCapability::ServiceDiscovery {
                protocols: vec!["http".to_string()],
            },
        ),
    ];

    // Test each capability by probing endpoints
    for (test_endpoints, capability) in capability_tests {
        for test_endpoint in test_endpoints {
            let test_url = format!("{endpoint}{test_endpoint}");

            match tokio::time::timeout(
                std::time::Duration::from_millis(2000),
                http_client.head(&test_url).send(),
            )
            .await
            {
                Ok(Ok(response)) => {
                    if response.status().is_success() || response.status().as_u16() == 405 {
                        // 405 Method Not Allowed means the endpoint exists but doesn't support HEAD
                        debug!("  ✅ Detected capability via endpoint: {}", test_endpoint);
                        capabilities.push(capability.clone());
                        break; // Found this capability, move to next
                    }
                }
                _ => {
                    // Endpoint not found or failed, continue to next
                }
            }
        }
    }

    capabilities
}

/// Infer primal type from discovered capabilities
fn infer_primal_type_from_capabilities(capabilities: &[PrimalCapability]) -> PrimalType {
    use PrimalCapability::*;

    // Count capability types to determine primary service type
    let mut security_caps = 0;
    let mut storage_caps = 0;
    let mut compute_caps = 0;
    let mut ai_caps = 0;
    let mut orchestration_caps = 0;
    let mut network_caps = 0;
    let mut custom_caps = 0;

    for capability in capabilities {
        match capability {
            Authentication { .. }
            | Encryption { .. }
            | ThreatDetection { .. }
            | Authorization { .. } => {
                security_caps += 1;
            }
            FileSystem { .. } | ObjectStorage { .. } | DataReplication { .. } | Backup { .. } => {
                storage_caps += 1;
            }
            ContainerRuntime { .. }
            | ServerlessExecution { .. }
            | LoadBalancing { .. }
            | AutoScaling { .. } => {
                compute_caps += 1;
            }
            ModelInference { .. }
            | AgentFramework { .. }
            | MachineLearning { .. }
            | NaturalLanguage { .. } => {
                ai_caps += 1;
            }
            Orchestration { .. } | ServiceDiscovery { .. } | Manifests { .. } => {
                orchestration_caps += 1;
            }
            NetworkRouting { .. } | ProxyServices { .. } | VpnServices { .. } => {
                network_caps += 1;
            }
            Custom { .. } => {
                custom_caps += 1;
            }
            _ => {}
        }
    }

    // Return type based on primary capability area
    if security_caps > 0
        && security_caps >= storage_caps
        && security_caps >= compute_caps
        && security_caps >= ai_caps
        && security_caps >= network_caps
        && security_caps >= orchestration_caps
        && security_caps >= custom_caps
    {
        PrimalType::from_string("security")
    } else if storage_caps > 0
        && storage_caps >= compute_caps
        && storage_caps >= ai_caps
        && storage_caps >= network_caps
        && storage_caps >= orchestration_caps
        && storage_caps >= custom_caps
    {
        PrimalType::from_string("storage")
    } else if compute_caps > 0
        && compute_caps >= ai_caps
        && compute_caps >= network_caps
        && compute_caps >= orchestration_caps
        && compute_caps >= custom_caps
    {
        PrimalType::from_string("compute")
    } else if ai_caps > 0
        && ai_caps >= network_caps
        && ai_caps >= orchestration_caps
        && ai_caps >= custom_caps
    {
        PrimalType::from_string("ai")
    } else if orchestration_caps > 0
        && orchestration_caps >= network_caps
        && orchestration_caps >= custom_caps
    {
        PrimalType::from_string("orchestration")
    } else if network_caps > 0 && network_caps >= custom_caps {
        PrimalType::from_string("network")
    } else if custom_caps > 0 {
        PrimalType::from_string("custom")
    } else {
        PrimalType::from_string("universal")
    }
}
