// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Shared tarpc RPC logic for [`super::TarpcServerSimple`] and [`super::TarpcServer`].

use std::collections::HashMap;
use std::sync::Arc;

use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_universal::tarpc_types::{
    HealthStatus, ProtocolInfo, RegistrationResult, ServiceInfo, ServiceRegistration, VersionInfo,
};
use tracing::{debug, info};

/// Discover services by capability (shared by both server types).
pub async fn discover(registry: &FederatedServiceRegistry, capability: String) -> Vec<ServiceInfo> {
    debug!("tarpc: discover(capability={})", capability);

    let services = registry.find_by_capability(&capability).await;
    debug!("Discovered {} services for capability '{}'", services.len(), capability);

    let cap: Arc<str> = Arc::from(capability);
    services
        .into_iter()
        .map(|svc| ServiceInfo {
            id: svc.service_id,
            capability: (*cap).to_string(),
            endpoint: svc.endpoint,
            status: svc.health_status.to_string(),
            metadata: None,
        })
        .collect()
}

/// List all services (shared).
pub async fn discover_all(registry: &FederatedServiceRegistry) -> Vec<ServiceInfo> {
    debug!("tarpc: discover_all()");

    let services = registry.get_all_services().await;
    debug!("Discovered {} total services", services.len());
    services
        .into_iter()
        .map(|svc| ServiceInfo {
            id: svc.service_id,
            capability: svc.service_type,
            endpoint: svc.endpoint,
            status: svc.health_status.to_string(),
            metadata: None,
        })
        .collect()
}

/// Register a service (shared).
pub async fn register(
    registry: &FederatedServiceRegistry,
    registration: ServiceRegistration,
) -> RegistrationResult {
    debug!("tarpc: register({}, {})", registration.service_id, registration.capability);

    let reg_id = registration.service_id;
    let reg_name = registration.service_name;
    let reg_cap = registration.capability;
    let capabilities = vec![reg_cap.clone()];

    let service_registration = songbird_network_federation::service_registry::ServiceRegistration {
        service_id: reg_id.clone(),
        service_name: reg_name.clone(),
        service_type: reg_cap,
        tower_id: registration.tower_id.unwrap_or_else(|| "unknown".to_string()),
        tower_name: registration.tower_name.unwrap_or_else(|| "Unknown Tower".to_string()),
        endpoint: registration.endpoint,
        capabilities,
        metadata: registration.metadata,
        health_status: songbird_network_federation::service_registry::ServiceHealthStatus::Healthy,
        registered_at: chrono::Utc::now(),
        last_seen: chrono::Utc::now(),
    };

    registry.register_local(service_registration).await;

    info!("Service registered: {} ({})", reg_name, reg_id);

    RegistrationResult {
        success: true,
        message: format!("Service {reg_id} registered successfully"),
    }
}

/// Unregister a service (shared). Log lines match the former `TarpcServerSimple` behavior.
pub async fn unregister(
    registry: &FederatedServiceRegistry,
    service_id: String,
) -> RegistrationResult {
    debug!("tarpc: unregister({})", service_id);

    let service_exists = registry.find_by_id(&service_id).await.is_some();

    if service_exists {
        registry.deregister_local(&service_id).await;
        info!("Service unregistered: {}", service_id);

        RegistrationResult {
            success: true,
            message: format!("Service {service_id} unregistered successfully"),
        }
    } else {
        debug!("Service not found for unregistration: {}", service_id);

        RegistrationResult {
            success: false,
            message: format!("Service {service_id} not found"),
        }
    }
}

/// Health snapshot (shared).
pub async fn health(
    registry: &FederatedServiceRegistry,
    start_time: std::time::Instant,
) -> HealthStatus {
    debug!("tarpc: health()");

    let uptime_seconds = start_time.elapsed().as_secs();
    let services_count = registry.get_all_services().await.len();

    HealthStatus {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds,
        services_count,
    }
}

/// Version info (shared).
pub fn version() -> VersionInfo {
    debug!("tarpc: version()");

    VersionInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        protocol: "tarpc".to_string(),
        capabilities: vec!["discovery".to_string(), "registry".to_string(), "health".to_string()],
    }
}

/// Protocol list for [`super::TarpcServerSimple`].
pub fn protocols_simple() -> Vec<ProtocolInfo> {
    use songbird_types::error_helpers::SafeEnv;

    debug!("tarpc: protocols()");

    let tarpc_port = SafeEnv::get_port("SONGBIRD_TARPC_PORT", 8081);
    let http_port = SafeEnv::get_port("SONGBIRD_HTTP_PORT", 8080);
    let ipc_path = crate::env_config::socket_path().to_string_lossy().into_owned();

    vec![
        ProtocolInfo {
            name: "tarpc".to_string(),
            port: tarpc_port,
            enabled: true,
            info: HashMap::new(),
        },
        ProtocolInfo {
            name: "jsonrpc".to_string(),
            port: 0,
            enabled: true,
            info: HashMap::from([("path".to_string(), ipc_path)]),
        },
        ProtocolInfo {
            name: "http".to_string(),
            port: http_port,
            enabled: true,
            info: HashMap::new(),
        },
    ]
}

/// Protocol list for legacy [`super::TarpcServer`].
pub fn protocols_legacy() -> Vec<ProtocolInfo> {
    debug!("tarpc: protocols()");
    use songbird_types::error_helpers::SafeEnv;

    let http_port = SafeEnv::get_port("SONGBIRD_HTTP_PORT", 8080);
    let https_port = SafeEnv::get_port("SONGBIRD_HTTPS_PORT", 8443);
    let tarpc_port = SafeEnv::get_port("SONGBIRD_TARPC_PORT", 8081);

    vec![
        ProtocolInfo {
            name: "HTTP".to_string(),
            port: http_port,
            enabled: true,
            info: HashMap::new(),
        },
        ProtocolInfo {
            name: "HTTPS".to_string(),
            port: https_port,
            enabled: true,
            info: HashMap::new(),
        },
        ProtocolInfo {
            name: "JSON-RPC".to_string(),
            port: https_port,
            enabled: true,
            info: HashMap::from([("path".to_string(), "/jsonrpc".to_string())]),
        },
        ProtocolInfo {
            name: "tarpc".to_string(),
            port: tarpc_port,
            enabled: true,
            info: HashMap::new(),
        },
    ]
}
