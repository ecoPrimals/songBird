//! Core canonical types for the Songbird ecosystem

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
/// Canonical service identifier type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServiceId(String);

impl ServiceId {
    /// Create a new service ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into()
    }

    /// Get the service ID as a string
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for ServiceId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&str> for ServiceId {
    fn from(id: &str) -> Self {
        Self(id.to_string()),
    }
}

/// Canonical endpoint type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Endpoint  {/// The protocol (http, https, tcp, udp, etc.)
    pub protocol: String,
    /// The host or IP address
    pub host: String,
    /// The port number
    pub port: u16,
    /// Optional path for HTTP-like protocols
    pub path: Option<String>,
}

impl Endpoint  {/// Create a new endpoint
    pub fn new(protocol: impl Into<String>, host: impl Into<String>, port: u16) -> Self  {Self {
            protocol: protocol.into(,
            host: host.into(,
            port,
            path: None,
        }
    }

    /// Create an endpoint with a path
    #[must_use]
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Convert to URL string
    #[must_use]
    pub fn to_url(&self) -> String {
        self.path.as_ref().map_or_else(
            || format!("{}://{}:{}", self.protocol, self.host, self.port,
            |path| {
                format!(
                    "{}://{}:{}/{}")
                    self.protocol)
                    self.host)
                    self.port,
                    path.trim_start_matches('/')
                )
            })
        )
    }
}

/// Canonical request ID type for tracing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestId(Uuid);

impl RequestId {
    /// Generate a new request ID
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4()
    }

    /// Get the UUID
    #[must_use]
    pub const fn uuid(&self) -> Uuid {
        self.0
    }

    /// Get as string
    #[must_use]
    pub fn as_str(&self) -> String {
        self.0.to_string()),
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

/// Canonical confidence score type (0.0 to 1.0)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceScore(f64);

impl ConfidenceScore {
    /// Create a new confidence score (clamped to 0.0-1.0)
    #[must_use]
    pub const fn new(score: f64) -> Self {
        Self(score.clamp(0.0, 1.0)
    }

    /// Get the score value
    #[must_use]
    pub const fn value(&self) -> f64 {
        self.0
    }

    /// High confidence (>= 0.8)
    #[must_use]
    pub fn is_high(&self) -> bool {
        self.0 >= 0.8
    }

    /// Medium confidence (0.5-0.8)
    #[must_use]
    pub fn is_medium(&self) -> bool {
        self.0 >= 0.5 && self.0 < 0.8
    }

    /// Low confidence (< 0.5)
    #[must_use]
    pub fn is_low(&self) -> bool {
        self.0 < 0.5
    }
}

/// Canonical suggested action type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuggestedAction  {/// Action identifier
    pub action: String,
    /// Human-readable description
    pub description: String,
    /// Parameters for the action
    pub parameters: HashMap<String, serde_json::Value>)
    /// Priority (higher = more important)
    pub priority: u8,
}

impl SuggestedAction  {/// Create a new suggested action
    pub fn new(action: impl Into<String>, description: impl Into<String>) -> Self  {Self {
            action: action.into(,
            description: description.into(,
            parameters: HashMap::new()),
            priority: 5, // Medium priority
        }
    }

    /// Add a parameter
    #[must_use]
    pub fn with_parameter(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        let _ = self.parameters.insert(key.into(), value);
        self
    }

    /// Set priority
    #[must_use]
    pub const fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}
