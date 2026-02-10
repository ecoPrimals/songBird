// SPDX-License-Identifier: AGPL-3.0-only
//! Coverage tests for songbird_discovery::traits::service and songbird_discovery::traits::discovery
//!
//! Tests struct construction, builder patterns, serialization, and defaults.

use songbird_discovery::traits::discovery::{
    DiscoveryBackend, DiscoveryConfig, ServiceEvent, ServiceHealthStatus, ServiceQuery,
    ServiceRegistration, SortBy,
};
use songbird_discovery::traits::service::{
    AuthInfo, ClientInfo, EndpointParameter, HealthStatus, ParameterType, ParameterValidation,
    RateLimit, ResponseStatus, ServiceEndpoint, ServiceInfo, ServiceMetrics, ServiceRequest,
    ServiceResponse, ServiceStatus,
};
use std::collections::HashMap;
use std::time::Duration;

// ═══════════════════════════════════════════════════════════════════════
// ServiceRequest tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_service_request_new() {
    let req = ServiceRequest::new("GET".to_string(), "/api/v1/health".to_string());
    assert_eq!(req.method, "GET");
    assert_eq!(req.path, "/api/v1/health");
    assert!(!req.id.is_empty());
    assert!(req.body.is_none());
    assert!(req.query_params.is_empty());
    assert!(req.client_info.is_none());
    assert!(req.auth_info.is_none());
    assert!(req.timeout.is_some());
    assert!(req.correlation_id.is_none());
    assert!(req.trace_id.is_none());
}

#[test]
fn test_service_request_with_header() {
    let req = ServiceRequest::new("POST".to_string(), "/api/submit".to_string())
        .with_header("Content-Type".to_string(), "application/json".to_string())
        .with_header("Authorization".to_string(), "Bearer token123".to_string());

    assert_eq!(req.headers.len(), 2);
    assert_eq!(req.headers.get("Content-Type").unwrap(), "application/json");
}

#[test]
fn test_service_request_with_body() {
    let body = serde_json::json!({"name": "test", "value": 42});
    let req =
        ServiceRequest::new("POST".to_string(), "/api/data".to_string()).with_body(body.clone());

    assert_eq!(req.body, Some(body));
}

#[test]
fn test_service_request_with_query_param() {
    let req = ServiceRequest::new("GET".to_string(), "/search".to_string())
        .with_query_param("q".to_string(), "rust".to_string())
        .with_query_param("page".to_string(), "1".to_string());

    assert_eq!(req.query_params.len(), 2);
    assert_eq!(req.query_params.get("q").unwrap(), "rust");
}

#[test]
fn test_service_request_with_client_info() {
    let client = ClientInfo {
        ip: Some("127.0.0.1:8080".parse().unwrap()),
        user_agent: Some("TestClient/1.0".to_string()),
        client_id: Some("client-abc".to_string()),
        session_id: None,
        request_count: Some(42),
    };

    let req = ServiceRequest::new("GET".to_string(), "/".to_string()).with_client_info(client);

    assert!(req.client_info.is_some());
    assert_eq!(req.client_info.as_ref().unwrap().request_count, Some(42));
}

#[test]
fn test_service_request_with_auth_info() {
    let auth = AuthInfo {
        user_id: Some("user-123".to_string()),
        roles: vec!["admin".to_string(), "editor".to_string()],
        permissions: vec!["read".to_string(), "write".to_string()],
        token_type: Some("Bearer".to_string()),
        expires_at: Some(chrono::Utc::now()),
        scopes: vec!["api:full".to_string()],
    };

    let req = ServiceRequest::new("GET".to_string(), "/".to_string()).with_auth_info(auth);

    assert!(req.auth_info.is_some());
    let ai = req.auth_info.unwrap();
    assert_eq!(ai.roles.len(), 2);
    assert_eq!(ai.permissions.len(), 2);
}

#[test]
fn test_service_request_serialization() {
    let req = ServiceRequest::new("GET".to_string(), "/api/test".to_string())
        .with_header("Accept".to_string(), "application/json".to_string())
        .with_query_param("key".to_string(), "value".to_string());

    let json = serde_json::to_string(&req).unwrap();
    let deserialized: ServiceRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.method, "GET");
    assert_eq!(deserialized.path, "/api/test");
    assert_eq!(deserialized.headers.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// ServiceResponse tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_service_response_success() {
    let resp = ServiceResponse::success("req-123".to_string());
    assert_eq!(resp.request_id, "req-123");
    assert_eq!(resp.status, ResponseStatus::Success);
    assert!(resp.error_message.is_none());
    assert!(resp.body.is_none());
}

#[test]
fn test_service_response_error() {
    let resp = ServiceResponse::error("req-456".to_string(), "Not found".to_string());
    assert_eq!(resp.status, ResponseStatus::Error);
    assert_eq!(resp.error_message.as_deref(), Some("Not found"));
}

#[test]
fn test_service_response_with_body() {
    let body = serde_json::json!({"result": "ok"});
    let resp = ServiceResponse::success("req-789".to_string()).with_body(body.clone());

    assert_eq!(resp.body, Some(body));
}

#[test]
fn test_service_response_with_header() {
    let resp = ServiceResponse::success("req-abc".to_string())
        .with_header("X-Request-Id".to_string(), "abc-123".to_string());

    assert_eq!(resp.headers.get("X-Request-Id").unwrap(), "abc-123");
}

#[test]
fn test_service_response_with_processing_time() {
    let resp = ServiceResponse::success("req-def".to_string())
        .with_processing_time(Duration::from_millis(150));

    assert_eq!(resp.processing_time, Duration::from_millis(150));
}

#[test]
fn test_response_status_variants() {
    let statuses = vec![
        ResponseStatus::Success,
        ResponseStatus::Error,
        ResponseStatus::Timeout,
        ResponseStatus::NotFound,
        ResponseStatus::Unauthorized,
        ResponseStatus::Forbidden,
    ];

    for status in &statuses {
        let json = serde_json::to_string(status).unwrap();
        let deserialized: ResponseStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(*status, deserialized);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ServiceInfo tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_service_info_serialization() {
    let info = ServiceInfo {
        service_id: "svc-001".to_string(),
        name: "auth-service".to_string(),
        version: "2.0.0".to_string(),
        service_type: "http".to_string(),
        description: Some("Authentication service".to_string()),
        endpoints: vec![ServiceEndpoint {
            path: "/auth/login".to_string(),
            method: "POST".to_string(),
            description: Some("Login endpoint".to_string()),
            parameters: vec![EndpointParameter {
                name: "username".to_string(),
                param_type: ParameterType::String,
                required: true,
                description: Some("User's username".to_string()),
                default_value: None,
                validation: Some(ParameterValidation {
                    min_length: Some(3),
                    max_length: Some(50),
                    min_value: None,
                    max_value: None,
                    pattern: None,
                    allowed_values: None,
                }),
            }],
            response_schema: None,
            auth_required: false,
            rate_limit: Some(RateLimit {
                requests_per_minute: 60,
                burst_size: Some(10),
                window_size: Duration::from_secs(60),
            }),
        }],
        health_check_endpoint: Some("/health".to_string()),
        metadata: HashMap::new(),
        tags: vec!["auth".to_string()],
        dependencies: vec!["db-service".to_string()],
        status: ServiceStatus::Running,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        instance_id: "inst-001".to_string(),
        host: "localhost".to_string(),
        port: 8080,
    };

    let json = serde_json::to_string(&info).unwrap();
    let deserialized: ServiceInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "auth-service");
    assert_eq!(deserialized.endpoints.len(), 1);
    assert_eq!(deserialized.port, 8080);
}

// ═══════════════════════════════════════════════════════════════════════
// ServiceStatus and HealthStatus
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_service_status_variants() {
    let statuses = vec![
        ServiceStatus::Starting,
        ServiceStatus::Running,
        ServiceStatus::Stopping,
        ServiceStatus::Stopped,
        ServiceStatus::Error,
        ServiceStatus::Maintenance,
    ];

    for status in &statuses {
        let json = serde_json::to_string(status).unwrap();
        let deserialized: ServiceStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(*status, deserialized);
    }
}

#[test]
fn test_health_status_variants() {
    let statuses = vec![
        HealthStatus::Healthy,
        HealthStatus::Degraded,
        HealthStatus::Unhealthy,
        HealthStatus::Unknown,
    ];

    for status in &statuses {
        let json = serde_json::to_string(status).unwrap();
        let deserialized: HealthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(*status, deserialized);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ServiceMetrics
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_service_metrics_serialization() {
    let metrics = ServiceMetrics {
        request_count: 10000,
        error_count: 50,
        average_response_time: 12.5,
        uptime: Duration::from_secs(86400),
        memory_usage: Some(1024 * 1024 * 512),
        cpu_usage: Some(25.5),
        active_connections: 150,
        custom_metrics: {
            let mut m = HashMap::new();
            m.insert("cache_hit_rate".to_string(), 0.95);
            m
        },
        queue_depth: 10,
        throughput_rps: 500.0,
        error_rate: 0.005,
        uptime_seconds: 86400,
        last_updated: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&metrics).unwrap();
    let deserialized: ServiceMetrics = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.request_count, 10000);
    assert_eq!(deserialized.active_connections, 150);
}

// ═══════════════════════════════════════════════════════════════════════
// ParameterType variants
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parameter_type_variants() {
    let types = vec![
        ParameterType::String,
        ParameterType::Integer,
        ParameterType::Float,
        ParameterType::Boolean,
        ParameterType::Array,
        ParameterType::Object,
        ParameterType::DateTime,
    ];

    for pt in &types {
        let json = serde_json::to_string(pt).unwrap();
        let deserialized: ParameterType = serde_json::from_str(&json).unwrap();
        let debug = format!("{:?}", deserialized);
        assert!(!debug.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ServiceQuery tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_service_query_default() {
    let query = ServiceQuery::default();
    assert!(query.name.is_none());
    assert!(query.service_id.is_none());
    assert!(query.service_type.is_none());
    assert!(query.version.is_none());
    assert!(query.tags.is_empty());
    assert!(query.metadata.is_empty());
    assert!(query.health_status.is_none());
    assert!(query.limit.is_none());
    assert!(query.sort_by.is_none());
}

#[test]
fn test_service_query_new() {
    let query = ServiceQuery::new();
    assert!(query.name.is_none());
}

#[test]
fn test_service_query_builder() {
    let query = ServiceQuery::new()
        .with_service_id("svc-001")
        .with_service_type("http")
        .with_version("1.0.0")
        .with_tag("production")
        .with_tag("primary")
        .with_metadata("region", serde_json::json!("us-east-1"))
        .with_health_status(songbird_discovery::traits::discovery::HealthStatus::Healthy)
        .with_limit(10)
        .sort_by(SortBy::Name);

    assert_eq!(query.service_id.as_deref(), Some("svc-001"));
    assert_eq!(query.service_type.as_deref(), Some("http"));
    assert_eq!(query.version.as_deref(), Some("1.0.0"));
    assert_eq!(query.tags.len(), 2);
    assert!(query.tags.contains(&"production".to_string()));
    assert_eq!(query.limit, Some(10));
    assert_eq!(query.sort_by, Some(SortBy::Name));
}

#[test]
fn test_service_query_serialization() {
    let query = ServiceQuery::new().with_service_type("grpc").with_tag("internal").with_limit(5);

    let json = serde_json::to_string(&query).unwrap();
    let deserialized: ServiceQuery = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.service_type.as_deref(), Some("grpc"));
    assert_eq!(deserialized.limit, Some(5));
}

// ═══════════════════════════════════════════════════════════════════════
// DiscoveryConfig tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_discovery_config_default() {
    let config = DiscoveryConfig::default();
    assert!(matches!(config.backend, DiscoveryBackend::Static));
    assert_eq!(config.health_check_interval, Duration::from_secs(30));
    assert_eq!(config.connection_timeout, Duration::from_secs(10));
    assert_eq!(config.retry_attempts, 3);
    assert_eq!(config.retry_delay, Duration::from_secs(1));
}

#[test]
fn test_discovery_backend_variants() {
    let backends = vec![
        DiscoveryBackend::Static,
        DiscoveryBackend::Songbird {
            federation_enabled: true,
            trust_verification: true,
            attribution_tracking: false,
        },
        DiscoveryBackend::Etcd {
            endpoints: vec!["http://etcd:2379".to_string()],
            username: Some("admin".to_string()),
            password: Some("secret".to_string()),
        },
        DiscoveryBackend::Kubernetes {
            namespace: Some("default".to_string()),
            in_cluster: true,
            kubeconfig_path: None,
        },
    ];

    for backend in &backends {
        let json = serde_json::to_string(backend).unwrap();
        let deserialized: DiscoveryBackend = serde_json::from_str(&json).unwrap();
        let debug = format!("{:?}", deserialized);
        assert!(!debug.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ServiceRegistration tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_service_registration_new() {
    let info = ServiceInfo {
        service_id: "svc-reg-001".to_string(),
        name: "test-service".to_string(),
        version: "1.0.0".to_string(),
        service_type: "http".to_string(),
        description: None,
        endpoints: vec![],
        health_check_endpoint: None,
        metadata: HashMap::new(),
        tags: vec![],
        dependencies: vec![],
        status: ServiceStatus::Starting,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        instance_id: "inst-001".to_string(),
        host: "localhost".to_string(),
        port: 9090,
    };

    let reg = ServiceRegistration::new(info.clone());
    assert!(reg.ttl.is_none());
    assert!(reg.health_check_interval.is_none());
    assert!(reg.tags.is_empty());
    assert!(reg.metadata.is_empty());
    assert_eq!(reg.service_info.name, "test-service");
}

#[test]
fn test_service_registration_with_ttl() {
    let info = ServiceInfo {
        service_id: "svc-reg-002".to_string(),
        name: "ttl-service".to_string(),
        version: "1.0.0".to_string(),
        service_type: "http".to_string(),
        description: None,
        endpoints: vec![],
        health_check_endpoint: None,
        metadata: HashMap::new(),
        tags: vec![],
        dependencies: vec![],
        status: ServiceStatus::Running,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        instance_id: "inst-002".to_string(),
        host: "localhost".to_string(),
        port: 9091,
    };

    let reg = ServiceRegistration::new(info)
        .with_ttl(Duration::from_secs(300))
        .with_health_check_interval(Duration::from_secs(15));

    assert_eq!(reg.ttl, Some(Duration::from_secs(300)));
    assert_eq!(reg.health_check_interval, Some(Duration::from_secs(15)));
}

// ═══════════════════════════════════════════════════════════════════════
// ServiceEvent tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_service_event_variants() {
    let events: Vec<ServiceEvent> = vec![
        ServiceEvent::ServiceUnregistered {
            service_id: "svc-001".to_string(),
        },
        ServiceEvent::ServiceHealthChanged {
            service_id: "svc-002".to_string(),
            health: ServiceHealthStatus::Healthy,
        },
        ServiceEvent::ServiceMetadataUpdated {
            service_id: "svc-003".to_string(),
        },
        ServiceEvent::NodeJoined {
            node_id: "node-001".to_string(),
        },
        ServiceEvent::NodeHealthChanged {
            node_id: "node-002".to_string(),
            health: ServiceHealthStatus::Degraded,
        },
    ];

    for event in &events {
        let json = serde_json::to_string(event).unwrap();
        let deserialized: ServiceEvent = serde_json::from_str(&json).unwrap();
        let debug = format!("{:?}", deserialized);
        assert!(!debug.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ServiceHealthStatus
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_service_health_status_serialization() {
    let statuses = vec![
        ServiceHealthStatus::Healthy,
        ServiceHealthStatus::Degraded,
        ServiceHealthStatus::Unhealthy,
        ServiceHealthStatus::Unknown,
    ];

    for status in &statuses {
        let json = serde_json::to_string(status).unwrap();
        let deserialized: ServiceHealthStatus = serde_json::from_str(&json).unwrap();
        let debug = format!("{:?}", deserialized);
        assert!(!debug.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SortBy
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_sort_by_variants() {
    let sorts = vec![SortBy::Name, SortBy::CreatedAt, SortBy::LastSeen, SortBy::Health];

    for sort in &sorts {
        let json = serde_json::to_string(sort).unwrap();
        let deserialized: SortBy = serde_json::from_str(&json).unwrap();
        assert_eq!(*sort, deserialized);
    }
}

