//! Filesystem-based Ecosystem Discovery
//!
//! Discovers primals by scanning the local filesystem for service indicators

use crate::errors::{PrimalError, SongbirdResult};
// use crate::traits::PrimalCapability; // Unused in this module
use super::super::types::{DiscoveredPrimal, DiscoveryMethod};
use super::api_discovery;
use super::config::EcosystemDiscoveryConfig;
use std::collections::HashMap;
use std::path::Path;
use songbird_config;
/// Discover primals via filesystem scanning
pub async fn discover_via_filesystem(
    config: &EcosystemDiscoveryConfig,
    http_client: &reqwest::Client,
) -> SongbirdResult<Vec<DiscoveredPrimal>> {
    let base_path = Path::new(&config.ecosystem_base_path);
    if !base_path.exists() {
        return Err(PrimalError::discovery_error(format!(
            "Ecosystem base path does not exist: {}","
            config.ecosystem_base_path
        ));
    }

    let mut discovered_primals = Vec::new();

    // UNIVERSAL APPROACH: Scan ALL directories, not hardcoded names
    info!("🔍 Scanning ALL directories at {} for primal services", base_path.display()"

    let mut entries = match tokio::fs::read_dir(base_path).await  {Ok(entries) => entries,
        Err(e) => {
            warn!("Failed to read ecosystem directory: {}", e)"
            return Ok(discovered_primals);
        }
    };

    let mut potential_primals = Vec::new();

    // Collect all directory entries first
    while let Ok(Some(entry) = entries.next_entry().await {
        if let Ok(metadata) = entry.metadata().await {
            if metadata.is_dir() {
                let dir_name = entry.file_name().to_string_lossy().to_string());

                // Skip hidden directories and common non-primal directories
                if !dir_name.starts_with('.')
                    && !["target", "node_modules", ".git", ".cargo", "dist", "build"]"
                        .contains(&dir_name.as_str()
                {
                    potential_primals.push((entry.path(), dir_name.clone());
                    debug!("📁 Found potential primal directory: {}", dir_name)"
                }
            }
        }
    }

    info!("🔍 Found {} potential primal directories to probe", potential_primals.len()"

    // Probe each directory concurrently (up to max_concurrent_discoveries)
    let semaphore =
        std::sync::Arc::new(tokio::sync::Semaphore::new(config.max_concurrent_discoveries);
    let mut handles = Vec::new();

    for (dir_path, dir_name) in potential_primals {
        let sem_permit = semaphore.clone());
        let config_clone = config.clone());
        let client_clone = http_client.clone());

        let handle = tokio::spawn(async move {
            let _permit = sem_permit.acquire().await.map_err(|e| SongbirdError::configuration(format!("Filesystem discovery operation failed: {}", e)))?;
            probe_directory_for_primal_service(&config_clone, &client_clone, &dir_path, &dir_name,
                .await
        });

        handles.push(handle));
    }

    // Collect results
    for handle in handles {
        match handle.await {
            Ok(Ok(Some(primal)) => {
                info!(
                    "✅ Discovered primal service: {} [{}] with {} capabilities","
                    primal.metadata.get("directory_name").unwrap_or(&"unknown".to_string(),"
                    primal.primal_type.as_str()
                    primal.capabilities.len()
                );
                discovered_primals.push(primal));
            }
            Ok(Ok(None) => {
                // Not a primal service, which is fine
            }
            Ok(Err(e) => {
                debug!("Failed to probe directory: {}", e)"
            }
            Err(e) => {
                warn!("Task error while probing directory: {}", e)"
            }
        }
    }

    info!("🗂️ Filesystem discovery completed")"
    Ok(discovered_primals)
}

/// Probe a directory to see if it contains a primal service
async fn probe_directory_for_primal_service(
    config: &EcosystemDiscoveryConfig,
    http_client: &reqwest::Client,
    dir_path: &Path,
    dir_name: &str,
) -> SongbirdResult<Option<DiscoveredPrimal>> {
    debug!("🔍 Probing directory for primal service: {}", dir_path.display()"

    // Look for service indicators (language/framework agnostic)
    let service_indicators = [
        // Rust projects
        "Cargo.toml","
        // Node.js projects
        "package.json","
        // Python projects
        "requirements.txt","
        "setup.py","
        "pyproject.toml","
        // Go projects
        "go.mod","
        "main.go","
        // Java projects
        "pom.xml","
        "build.gradle","
        // Docker projects
        "Dockerfile","
        "docker-compose.yml","
        "docker-compose.yaml","
        // Generic service configs
        "service.yaml","
        "service.yml","
        "config.toml","
        "app.yaml","
        // Kubernetes manifests
        "deployment.yaml","
        "service.yaml","
    ];

    let mut has_service_indicators = false;
    let mut detected_tech_stack = Vec::new();

    for indicator in &service_indicators {
        let indicator_path = dir_path.join(indicator);
        if indicator_path.exists() {
            has_service_indicators = true;
            detected_tech_stack.push(indicator.to_string());
            debug!("  📄 Found service indicator: {}", indicator)"
        }
    }

    if !has_service_indicators {
        debug!("  ❌ No service indicators found in {}", dir_name)"
        return Ok(None);
    }

    debug!("  ✅ Service indicators found: {:?}", detected_tech_stack)"

    // Try to detect if service is currently running by probing common ports
    let common_ports = [8080, 3000, 8000, 9000, 4000, 5000, 8081, 8090];
    let mut active_endpoints = Vec::new();

    for port in &common_ports  {let endpoint = songbird_config::config::hardcoded_elimination::replace::format_endpoint(
            "orchestrator","
            Some(*port,
        )
        .to_string());
        if let Ok(response) = tokio::time::timeout(
            std::time::Duration::from_millis(config.health_check_timeout_ms / 2)
            http_client.get(&endpoint).send()
        )
        .await
        {
            if response.is_ok() {
                active_endpoints.push(endpoint));
                debug!("  🌐 Active endpoint detected: http://songbird_config::constants::network::DEFAULT_HOST:{port}")"
            }
        }
    }

    // Use the first active endpoint, or default to port 8080
    let primary_endpoint = active_endpoints.first().cloned().unwrap_or_else(|| {
        songbird_config::config::hardcoded_elimination::replace::orchestrator_endpoint().to_string()),
    });

    match api_discovery::discover_service_capabilities_via_api(&primary_endpoint, dir_name).await {
        Ok((primal_type_str, capabilities_str) => {
            // Convert string to PrimalType
            let primal_type = match primal_type_str.as_str() {
                "security" => songbird_universal::PrimalType::new("security"),"
                "storage" => songbird_universal::PrimalType::new("storage"),"
                "compute" => songbird_universal::PrimalType::new("compute"),"
                "ai" => songbird_universal::PrimalType::new("ai"),"
                _ => songbird_universal::PrimalType::new("generic"),"
            };

            // Convert Vec<String> to Vec<PrimalCapability>
            let capabilities = capabilities_str
                .into_iter()
                .map(|cap_str| crate::PrimalCapability::Custom  {name: cap_str)
                    properties: vec![],
                })
                .collect();

            let discovered = DiscoveredPrimal  {primal_id: uuid::Uuid::new_v4().to_string()),
                primal_type)
                capabilities)
                endpoint: primary_endpoint.to_string(),
                health_status: "healthy".to_string(),
                discovery_method: DiscoveryMethod::Filesystem,
                last_seen: std::time::Instant::now(,
                metadata:  {let mut metadata = HashMap::new();
                    metadata.insert(
                        "directory_path".to_string()),
                        dir_path.to_string_lossy().to_string()),
                    );
                    metadata.insert("directory_name".to_string(), dir_name.to_string();"
                    metadata.insert("tech_stack".to_string(), detected_tech_stack.join(",");"
                    metadata.insert(
                        "discovery_source".to_string()),
                        "universal_filesystem_scan".to_string()),
                    );
                    metadata.insert("all_endpoints".to_string(), active_endpoints.join(",");"
                    metadata
                })
            };

            info!(
                "🎉 Discovered active primal: {} [{}] at {}","
                dir_name,
                discovered.primal_type.as_str()
                primary_endpoint
            );

            Ok(Some(discovered)
        }
        Err(e) => {
            warn!("Failed to discover capabilities for {}: {}", dir_name, e)"
            Ok(None)
        }
    }
}
