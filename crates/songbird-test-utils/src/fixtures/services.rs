//! Service Registration Test Fixtures
//!
//! Provides helper functions to create test service registrations.

use serde::{Deserialize, Serialize};

/// Test service registration builder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestService {
    /// Service ID
    pub id: String,
    /// Service name
    pub name: String,
    /// Service endpoint
    pub endpoint: String,
    /// Service capabilities
    pub capabilities: Vec<String>,
    /// Service metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl TestService {
    /// Create a new test service
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name,
            endpoint: format!(
                // Test uses localhost - acceptable for unit tests
                "http://localhost:{}",
                fastrand::u16(10000..60000)
            ),
            capabilities: vec![],
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Add a capability
    #[must_use]
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    /// Add multiple capabilities
    #[must_use]
    pub fn with_capabilities(mut self, capabilities: Vec<impl Into<String>>) -> Self {
        self.capabilities.extend(capabilities.into_iter().map(Into::into));
        self
    }

    /// Set a specific endpoint
    #[must_use]
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    /// Add metadata
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get service ID
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get service name
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get endpoint
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Get capabilities
    #[must_use]
    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }
}

/// Create a compute service fixture
pub fn compute_service(name: impl Into<String>) -> TestService {
    TestService::new(name).with_capability("compute").with_metadata("type", "toadstool")
}

/// Create a storage service fixture
pub fn storage_service(name: impl Into<String>) -> TestService {
    TestService::new(name).with_capability("storage").with_metadata("type", "nestgate")
}

/// Create a security service fixture
pub fn security_service(name: impl Into<String>) -> TestService {
    TestService::new(name)
        .with_capability("security")
        .with_capability("auth")
        .with_metadata("type", "beardog")
}

/// Create an AI service fixture
pub fn ai_service(name: impl Into<String>) -> TestService {
    TestService::new(name)
        .with_capability("ai")
        .with_capability("inference")
        .with_metadata("type", "squirrel")
}

/// Create a multi-capability service fixture
pub fn multi_capability_service(
    name: impl Into<String>,
    capabilities: Vec<impl Into<String>>,
) -> TestService {
    TestService::new(name).with_capabilities(capabilities)
}

#[cfg(test)]
#[allow(clippy::uninlined_format_args)]
#[allow(clippy::float_cmp)]
#[allow(clippy::useless_vec)]
#[allow(clippy::unreadable_literal)]
#[allow(clippy::items_after_statements)]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
mod tests {
    #![allow(clippy::all)]
    #![allow(unused)]

    use super::*;

    #[test]
    fn test_service_builder() {
        let service = TestService::new("test-service")
            .with_capability("compute")
            .with_capability("storage")
            .with_metadata("region", "us-east-1");

        assert_eq!(service.name(), "test-service");
        assert_eq!(service.capabilities().len(), 2);
        assert!(service.capabilities().contains(&"compute".to_string()));
        assert_eq!(service.metadata.get("region"), Some(&"us-east-1".to_string()));
    }

    #[test]
    fn test_compute_service() {
        let service = compute_service("toadstool-1");
        assert!(service.capabilities().contains(&"compute".to_string()));
        assert_eq!(service.metadata.get("type"), Some(&"toadstool".to_string()));
    }

    #[test]
    fn test_storage_service() {
        let service = storage_service("nestgate-1");
        assert!(service.capabilities().contains(&"storage".to_string()));
        assert_eq!(service.metadata.get("type"), Some(&"nestgate".to_string()));
    }

    #[test]
    fn test_security_service() {
        let service = security_service("beardog-1");
        assert!(service.capabilities().contains(&"security".to_string()));
        assert!(service.capabilities().contains(&"auth".to_string()));
    }

    #[test]
    fn test_ai_service() {
        let service = ai_service("squirrel-1");
        assert!(service.capabilities().contains(&"ai".to_string()));
        assert!(service.capabilities().contains(&"inference".to_string()));
    }
}
