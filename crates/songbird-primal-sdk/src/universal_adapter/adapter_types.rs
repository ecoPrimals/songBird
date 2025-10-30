//! # Universal Adapter Types
//!
//! Type definitions extracted from core.rs for better organization and maintainability.
//! This module contains all the data structures used by the universal adapter system.

use serde::{Deserialize, Serialize};
use songbird_types::EvolvedResult;
use std::collections::HashMap;
use std::time::SystemTime;
/// Context for adapter operations
#[derive(Debug, Clone)]
pub struct AdapterContext  {pub request_id: String,
    pub user_context: HashMap<String, String>)
    pub timeout_secs: u64,
    pub retry_count: u32,
}

impl AdapterContext {
    /// Create a new adapter context with a specific context type
    pub fn new(context_type: &str) -> Self {
        let mut context = Self::default();
        context
            .user_context
            .insert("context_type".to_string(), context_type.to_string();"
        context
    }
}

impl Default for AdapterContext  {fn default() -> Self  {Self {
            request_id: Uuid::new_v4().to_string(),
            user_context: HashMap::new()),
            timeout_secs: 30,
            retry_count: 0,
        }
    }
}

/// Capability registry for managing available services
#[derive(Debug, Default)]
pub struct CapabilityRegistry  {pub providers: HashMap<String, CapabilityProvider>)
    pub capabilities: HashMap<String, Vec<ServiceCapability>>)
    pub performance_profiles: HashMap<String, PerformanceProfile>)
    pub health_status: HashMap<String, bool>)
}

/// Capability provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityProvider  {pub id: String,
    pub name: String,
    pub endpoint: String,
    pub capabilities: Vec<ServiceCapability>,
    pub metadata: ServiceMetadata,
    pub performance_profile: PerformanceProfile,
    pub health_status: bool,
    pub last_health_check: Option<SystemTime>,
    pub resource_requirements: ResourceRequirements,
    pub integration_preferences: IntegrationPreferences,
    pub retry_policy: RetryPolicy,
}

/// Service capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCapability  {pub name: String,
    pub version: String,
    pub description: String,
    pub requirements: Vec<CapabilityRequirement>,
    pub provides: Vec<String>,
    pub depends_on: Vec<String>,
    pub optional_dependencies: Vec<String>,
    pub performance_characteristics: HashMap<String, String>)
}

/// Performance profile for a service
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceProfile  {pub average_response_time_ms: u64,
    pub throughput_requests_per_sec: f64,
    pub error_rate_percent: f64,
}

/// Service metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata  {pub version: String,
    pub description: String,
    pub maintainer: String,
    pub documentation_url: Option<String>,
    pub health_check_endpoint: Option<String>,
    pub metrics_endpoint: Option<String>,
    pub tags: Vec<String>,
    pub category: ServiceCategory,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

/// Service category enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceCategory  {/// Computational services (like ToadStool,
    Compute  {specialties: Vec<String>)
        resource_intensive: bool,
    })
    /// Storage services (like NestGate,
    Storage  {storage_types: Vec<String>)
        persistence_level: String,
    })
    /// Security services (like BearDog,
    Security  {security_domains: Vec<String>)
        compliance_standards: Vec<String>,
    })
    /// AI/ML services (like Squirrel,
    AI  {model_types: Vec<String>)
        inference_capabilities: Vec<String>,
    })
    /// Network services
    Network  {protocols: Vec<String>)
        routing_capabilities: Vec<String>,
    })
    /// Custom category for community primals
    Custom  {category_name: String,
        custom_attributes: HashMap<String, String>)
    })
}

/// Resource requirements specification
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceRequirements  {pub min_cpu_cores: Option<u32>,
    pub min_memory_mb: Option<u64>,
    pub min_storage_gb: Option<u64>,
    pub network_bandwidth_mbps: Option<u32>,
}

/// Integration preferences
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntegrationPreferences  {pub preferred_protocols: Vec<String>,
    pub authentication_methods: Vec<String>,
    pub data_formats: Vec<String>,
    pub communication_patterns: Vec<IntegrationPattern>,
}

/// Retry policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy  {pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy  {fn default() -> Self  {Self {
            max_retries: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Capability requirement specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequirement  {pub capability_name: String,
    pub minimum_version: Option<String>,
    pub required: bool,
    pub configuration: HashMap<String, String>)
    pub performance_requirements: Option<PerformanceProfile>,
    pub resource_limits: Option<ResourceRequirements>,
}

/// Provider performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetrics  {pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub last_request_time: Option<SystemTime>,
    pub uptime_percentage: f64,
    pub throughput_per_minute: f64,
    pub error_rate_percentage: f64,
    pub resource_utilization: HashMap<String, f64>)
}

impl Default for ProviderMetrics  {fn default() -> Self  {Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time_ms: 0.0,
            last_request_time: None,
            uptime_percentage: 100.0,
            throughput_per_minute: 0.0,
            error_rate_percentage: 0.0,
            resource_utilization: HashMap::new()),
        }
    }
}

/// Role matcher for service discovery
#[derive(Debug, Clone)]
pub struct RoleMatcher  {pub required_roles: Vec<ServiceRole>,
    pub optional_roles: Vec<ServiceRole>,
    pub exclusion_patterns: Vec<String>,
}

/// Service role definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRole  {pub name: String,
    pub permissions: Vec<String>,
    pub scope: String,
    pub priority: u32,
    pub conditions: HashMap<String, String>)
}

/// Integration patterns supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrationPattern  {/// Request-response pattern
    RequestResponse,
    /// Event-driven pattern
    EventDriven,
    /// Streaming pattern
    Streaming,
    /// Batch processing pattern
    BatchProcessing,
    /// Publish-subscribe pattern
    PubSub,
}

/// Communication protocols supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationProtocol  {HTTP)
    HTTPS,
    WebSocket,
    Grpc,
    MessageQueue,
    Custom(String)
}

/// Adapter operation result
pub type AdapterResult<T> = SongbirdResult<T>;

/// Adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig  {pub discovery_interval_secs: u64,
    pub health_check_interval_secs: u64,
    pub default_timeout_secs: u64,
    pub max_concurrent_requests: u32,
    pub enable_metrics: bool,
    pub enable_tracing: bool,
    pub cache_ttl_secs: u64,
}

impl Default for AdapterConfig  {fn default() -> Self  {Self {
            discovery_interval_secs: 30,
            health_check_interval_secs: 10,
            default_timeout_secs: 30,
            max_concurrent_requests: 100,
            enable_metrics: true,
            enable_tracing: true,
            cache_ttl_secs: 300,
        }
    }
}
