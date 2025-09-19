//! Type definitions for Universal Primals
//!
//! Provides core types and data structures for the Universal Primals system
//! with modern Rust idioms and comprehensive documentation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Security level for primal operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SecurityLevel {
    /// Basic user-level security
    User,
    /// Standard security level
    Standard,
    /// High security level
    High,
    /// Maximum security level
    Maximum,
}

impl Default for SecurityLevel {
    fn default() -> Self {
        Self::Standard
    }
}

/// Network location information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NetworkLocation {
    /// Network region
    pub region: String,
    /// Availability zone
    pub zone: Option<String>,
    /// Network latency in milliseconds
    pub latency_ms: Option<u64>,
    /// Bandwidth capacity
    pub bandwidth_mbps: Option<u64>,
}

impl Default for NetworkLocation {
    fn default() -> Self {
        Self {
            region: "unknown".to_string(),
            zone: None,
            latency_ms: None,
            bandwidth_mbps: None,
        }
    }
}

/// Dynamic port information for primal services
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DynamicPortInfo {
    /// Port number
    pub port: u16,
    /// Protocol (e.g., "http", "grpc", "tcp")
    pub protocol: String,
    /// Host address
    pub host: String,
    /// Optional service path
    pub path: Option<String>,
    /// Whether TLS is enabled
    pub tls_enabled: bool,
}

impl DynamicPortInfo {
    /// Create new dynamic port info
    #[must_use]
    pub fn new(port: u16, protocol: impl Into<String>, host: impl Into<String>) -> Self {
        Self {
            port,
            protocol: protocol.into(),
            host: host.into(),
            path: None,
            tls_enabled: false,
        }
    }

    /// Set the service path
    #[must_use]
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Enable TLS
    #[must_use]
    pub fn with_tls(mut self) -> Self {
        self.tls_enabled = true;
        self
    }

    /// Get the full URL for this service
    #[must_use]
    pub fn url(&self) -> String {
        let scheme = if self.tls_enabled {
            match self.protocol.as_str() {
                "http" => "https",
                "grpc" => "grpcs",
                _ => &self.protocol,
            }
        } else {
            &self.protocol
        };

        let path = self.path.as_deref().unwrap_or("");
        format!("{}://{}:{}{}", scheme, self.host, self.port, path)
    }
}

/// Primal service endpoints configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrimalEndpoints {
    /// Primary endpoint
    pub primary: String,
    /// Backup endpoints
    pub backup: Vec<String>,
    /// Health check endpoint
    pub health_check: Option<String>,
    /// Metrics endpoint
    pub metrics: Option<String>,
}

impl PrimalEndpoints {
    /// Create new primal endpoints
    #[must_use]
    pub fn new(primary: impl Into<String>) -> Self {
        Self {
            primary: primary.into(),
            backup: Vec::new(),
            health_check: None,
            metrics: None,
        }
    }

    /// Add a backup endpoint
    #[must_use]
    pub fn add_backup(mut self, endpoint: impl Into<String>) -> Self {
        self.backup.push(endpoint.into());
        self
    }

    /// Set health check endpoint
    #[must_use]
    pub fn with_health_check(mut self, endpoint: impl Into<String>) -> Self {
        self.health_check = Some(endpoint.into());
        self
    }

    /// Set metrics endpoint
    #[must_use]
    pub fn with_metrics(mut self, endpoint: impl Into<String>) -> Self {
        self.metrics = Some(endpoint.into());
        self
    }

    /// Get all endpoints (primary + backup)
    #[must_use]
    pub fn all_endpoints(&self) -> Vec<&str> {
        let mut endpoints = vec![self.primary.as_str()];
        endpoints.extend(self.backup.iter().map(String::as_str));
        endpoints
    }
}

/// Context information for primal services
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrimalContext {
    /// User identifier
    pub user_id: Option<String>,
    /// Organization identifier
    pub org_id: Option<String>,
    /// Device identifier
    pub device_id: Option<String>,
    /// Session identifier
    pub session_id: Option<String>,
    /// Request identifier for tracing
    pub request_id: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Default for PrimalContext {
    fn default() -> Self {
        Self {
            user_id: None,
            org_id: None,
            device_id: None,
            session_id: None,
            request_id: None,
            metadata: HashMap::new(),
        }
    }
}

impl PrimalContext {
    /// Create a new context
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set user ID
    #[must_use]
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set organization ID
    #[must_use]
    pub fn with_org_id(mut self, org_id: impl Into<String>) -> Self {
        self.org_id = Some(org_id.into());
        self
    }

    /// Set device ID
    #[must_use]
    pub fn with_device_id(mut self, device_id: impl Into<String>) -> Self {
        self.device_id = Some(device_id.into());
        self
    }

    /// Set session ID
    #[must_use]
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Set request ID
    #[must_use]
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Add metadata
    #[must_use]
    pub fn add_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if context has required user information
    #[must_use]
    pub fn has_user_info(&self) -> bool {
        self.user_id.is_some() || self.device_id.is_some()
    }
}

impl fmt::Display for PrimalContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PrimalContext(")?;

        let mut parts = Vec::new();
        if let Some(user_id) = &self.user_id {
            parts.push(format!("user:{}", user_id));
        }
        if let Some(org_id) = &self.org_id {
            parts.push(format!("org:{}", org_id));
        }
        if let Some(device_id) = &self.device_id {
            parts.push(format!("device:{}", device_id));
        }
        if let Some(session_id) = &self.session_id {
            parts.push(format!("session:{}", session_id));
        }

        write!(f, "{}", parts.join(", "))?;
        write!(f, ")")
    }
}

/// Configuration for primal services
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrimalConfig {
    /// Instance-specific configuration
    pub instance: HashMap<String, serde_json::Value>,
    /// Security configuration
    pub security: HashMap<String, serde_json::Value>,
    /// Performance tuning configuration
    pub performance: HashMap<String, serde_json::Value>,
    /// Custom configuration
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for PrimalConfig {
    fn default() -> Self {
        Self {
            instance: HashMap::new(),
            security: HashMap::new(),
            performance: HashMap::new(),
            custom: HashMap::new(),
        }
    }
}

impl PrimalConfig {
    /// Create a new configuration
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set an instance configuration value
    #[must_use]
    pub fn set_instance(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.instance.insert(key.into(), value);
        self
    }

    /// Set a security configuration value
    #[must_use]
    pub fn set_security(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.security.insert(key.into(), value);
        self
    }

    /// Set a performance configuration value
    #[must_use]
    pub fn set_performance(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.performance.insert(key.into(), value);
        self
    }

    /// Set a custom configuration value
    #[must_use]
    pub fn set_custom(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.custom.insert(key.into(), value);
        self
    }

    /// Get an instance configuration value
    #[must_use]
    pub fn get_instance(&self, key: &str) -> Option<&serde_json::Value> {
        self.instance.get(key)
    }

    /// Get a security configuration value
    #[must_use]
    pub fn get_security(&self, key: &str) -> Option<&serde_json::Value> {
        self.security.get(key)
    }

    /// Get a performance configuration value
    #[must_use]
    pub fn get_performance(&self, key: &str) -> Option<&serde_json::Value> {
        self.performance.get(key)
    }

    /// Get a custom configuration value
    #[must_use]
    pub fn get_custom(&self, key: &str) -> Option<&serde_json::Value> {
        self.custom.get(key)
    }

    /// Merge with another configuration
    pub fn merge(&mut self, other: PrimalConfig) {
        self.instance.extend(other.instance);
        self.security.extend(other.security);
        self.performance.extend(other.performance);
        self.custom.extend(other.custom);
    }

    /// Check if configuration is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.instance.is_empty()
            && self.security.is_empty()
            && self.performance.is_empty()
            && self.custom.is_empty()
    }
}
