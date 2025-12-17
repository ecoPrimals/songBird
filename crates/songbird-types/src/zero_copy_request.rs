//! Optimized types for request routing with zero-copy patterns
//!
//! This module provides Arc<str> based types to eliminate cloning in hot paths.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// HTTP header key constants (zero allocation)
pub mod header_keys {
    /// Request path header
    pub const X_REQUEST_PATH: &str = "x-request-path";
    /// Request method header
    pub const X_REQUEST_METHOD: &str = "x-request-method";
    /// Target service header
    pub const X_TARGET_SERVICE: &str = "x-target-service";
    /// Trace ID header
    pub const X_TRACE_ID: &str = "x-trace-id";
    /// Orchestrator timestamp header
    pub const X_ORCHESTRATOR_TIMESTAMP: &str = "x-orchestrator-timestamp";
    /// Authorization header
    pub const AUTHORIZATION: &str = "authorization";
    /// Content type header
    pub const CONTENT_TYPE: &str = "content-type";
}

/// Zero-copy optimized service request
///
/// Uses Arc<str> for strings that are frequently cloned across async boundaries.
/// This provides cheap cloning (atomic increment) instead of deep string copies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroCopyServiceRequest {
    /// Request ID (Arc for zero-copy sharing)
    #[serde(with = "arc_str_serde")]
    pub id: Arc<str>,

    /// Service ID (Arc for zero-copy sharing)
    #[serde(with = "arc_str_serde")]
    pub service_id: Arc<str>,

    /// Request path (Arc for zero-copy sharing)
    #[serde(with = "arc_str_serde")]
    pub path: Arc<str>,

    /// HTTP method (Arc for zero-copy sharing)
    #[serde(with = "arc_str_serde")]
    pub method: Arc<str>,

    /// Request payload (already behind Arc in serde_json::Value)
    pub payload: serde_json::Value,

    /// Request headers (values are Arc<str> for zero-copy)
    #[serde(with = "arc_str_hashmap_serde")]
    pub headers: HashMap<Arc<str>, Arc<str>>,

    /// Request timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Request timeout
    pub timeout: Option<std::time::Duration>,

    /// Request metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ZeroCopyServiceRequest {
    /// Create a new zero-copy service request
    pub fn new(
        id: impl Into<String>,
        service_id: impl Into<String>,
        path: impl Into<String>,
        method: impl Into<String>,
    ) -> Self {
        Self {
            id: Arc::from(id.into().as_str()),
            service_id: Arc::from(service_id.into().as_str()),
            path: Arc::from(path.into().as_str()),
            method: Arc::from(method.into().as_str()),
            payload: serde_json::Value::Null,
            headers: HashMap::new(),
            timestamp: chrono::Utc::now(),
            timeout: None,
            metadata: HashMap::new(),
        }
    }

    /// Add a header (zero-copy after initial allocation)
    pub fn with_header(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        let key_arc = Arc::from(key.into().as_str());
        let value_arc = Arc::from(value.into().as_str());
        self.headers.insert(key_arc, value_arc);
        self
    }

    /// Get header value (cheap Arc clone if needed)
    pub fn get_header(&self, key: &str) -> Option<Arc<str>> {
        self.headers.iter().find(|(k, _)| k.as_ref() == key).map(|(_, v)| Arc::clone(v))
    }
}

/// Serde support for Arc<str>
mod arc_str_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::sync::Arc;

    pub fn serialize<S>(arc: &Arc<str>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(arc)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Arc<str>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Arc::from(s.as_str()))
    }
}

/// Serde support for HashMap<Arc<str>, Arc<str>>
mod arc_str_hashmap_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::HashMap;
    use std::sync::Arc;

    pub fn serialize<S>(map: &HashMap<Arc<str>, Arc<str>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string_map: HashMap<&str, &str> =
            map.iter().map(|(k, v)| (k.as_ref(), v.as_ref())).collect();
        string_map.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<Arc<str>, Arc<str>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string_map = HashMap::<String, String>::deserialize(deserializer)?;
        Ok(string_map
            .into_iter()
            .map(|(k, v)| (Arc::from(k.as_str()), Arc::from(v.as_str())))
            .collect())
    }
}

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_copy_request_creation() {
        let req =
            ZeroCopyServiceRequest::new("req-123", "compute-service", "/api/v1/compute", "POST");

        assert_eq!(req.id.as_ref(), "req-123");
        assert_eq!(req.service_id.as_ref(), "compute-service");
        assert_eq!(req.path.as_ref(), "/api/v1/compute");
        assert_eq!(req.method.as_ref(), "POST");
    }

    #[test]
    fn test_arc_cloning_is_cheap() {
        let req = ZeroCopyServiceRequest::new("req-123", "service", "/path", "GET");

        // Arc clone only increments atomic counter, no deep copy
        let id_clone = Arc::clone(&req.id);
        assert_eq!(id_clone.as_ref(), "req-123");

        // Both point to same data
        assert!(Arc::ptr_eq(&req.id, &id_clone));
    }

    #[test]
    fn test_header_operations() {
        let mut req = ZeroCopyServiceRequest::new("req-123", "service", "/path", "GET");

        req.with_header("authorization", "Bearer token123");
        req.with_header("content-type", "application/json");

        assert_eq!(req.get_header("authorization").unwrap().as_ref(), "Bearer token123");
        assert_eq!(req.get_header("content-type").unwrap().as_ref(), "application/json");
    }

    #[test]
    fn test_serde_roundtrip() {
        let mut req = ZeroCopyServiceRequest::new("req-123", "service", "/path", "POST");
        req.with_header("test-header", "test-value");

        // Serialize
        let json = serde_json::to_string(&req).unwrap();

        // Deserialize
        let deserialized: ZeroCopyServiceRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id.as_ref(), "req-123");
        assert_eq!(deserialized.service_id.as_ref(), "service");
        assert_eq!(deserialized.get_header("test-header").unwrap().as_ref(), "test-value");
    }

    #[test]
    fn test_zero_copy_vs_string_clone() {
        // This test demonstrates the memory benefit
        let original: Arc<str> = Arc::from("very-long-request-id-that-would-be-expensive-to-clone");

        // Zero-copy Arc clone (just atomic increment)
        let clone1 = Arc::clone(&original);
        let clone2 = Arc::clone(&original);
        let clone3 = Arc::clone(&original);

        // All point to same memory
        assert!(Arc::ptr_eq(&original, &clone1));
        assert!(Arc::ptr_eq(&original, &clone2));
        assert!(Arc::ptr_eq(&original, &clone3));

        // String clone would allocate 4 separate copies
        // Arc clone: 4 pointers + 1 shared string = O(1) overhead
        // String clone: 4 separate strings = O(4n) overhead
    }
}
