//! Capability request and response types

use songbird_errors::EvolvedResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Removed unused SongbirdResponse import
/// Request sent to a primal capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequest {
    pub request_id: Uuid,
    pub capability_type: String,
    pub operation: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub timeout_ms: Option<u64>,
}

impl CapabilityRequest {
    /// Create a new capability request
    pub fn new(capability_type: String, operation: String) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            capability_type,
            operation,
            parameters: HashMap::new(),
            timeout_ms: Some(30000), // 30 seconds default
        }
    }

    /// Create a request with parameters
    pub fn with_parameters(
        capability_type: String,
        operation: String,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            capability_type,
            operation,
            parameters,
            timeout_ms: Some(30000),
        }
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    /// Add a parameter to the request
    pub fn add_parameter(mut self, key: String, value: serde_json::Value) -> Self {
        self.parameters.insert(key, value);
        self
    }

    /// Get a parameter from the request
    pub fn get_parameter(&self, key: &str) -> Option<&serde_json::Value> {
        self.parameters.get(key)
    }

    /// Get a string parameter
    pub fn get_string_parameter(&self, key: &str) -> Option<String> {
        self.parameters
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Get a numeric parameter
    pub fn get_number_parameter(&self, key: &str) -> Option<f64> {
        self.parameters.get(key).and_then(|v| v.as_f64())
    }

    /// Get a boolean parameter
    pub fn get_bool_parameter(&self, key: &str) -> Option<bool> {
        self.parameters.get(key).and_then(|v| v.as_bool())
    }

    /// Check if request has a specific parameter
    pub fn has_parameter(&self, key: &str) -> bool {
        self.parameters.contains_key(key)
    }

    /// Validate request has required parameters
    pub fn validate_required_parameters(&self, required: &[&str]) -> SongbirdResult<()> {
        for param in required {
            if !self.has_parameter(param) {
                return Err(SongbirdError::internal_error(internal_error("Missing required parameter: {param}"));
            }
        }
        Ok(SongbirdResponse::success(()))
    }
}

/// Response structure for capability operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityResponse {
    pub request_id: Uuid,
    pub success: bool,
    pub data: serde_json::Value,
    pub error_message: Option<String>,
    pub execution_time_ms: Option<u64>,
    pub provider_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl CapabilityResponse {
    /// Create a successful response
    pub fn success(request_id: Uuid, data: serde_json::Value) -> Self {
        Self {
            request_id,
            success: true,
            data,
            error_message: None,
            execution_time_ms: None,
            provider_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Create an error response
    pub fn error(request_id: Uuid, error_message: String) -> Self {
        Self {
            request_id,
            success: false,
            data: serde_json::Value::Null,
            error_message: Some(error_message),
            execution_time_ms: None,
            provider_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Set execution time
    pub fn with_execution_time(mut self, execution_time_ms: u64) -> Self {
        self.execution_time_ms = Some(execution_time_ms);
        self
    }

    /// Set provider ID
    pub fn with_provider_id(mut self, provider_id: String) -> Self {
        self.provider_id = Some(provider_id);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Get data as a specific type
    pub fn get_data<T>(&self) -> Result<T, serde_json::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        serde_json::from_value(self.data.clone())
    }

    /// Check if response is successful
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Check if response is an error
    pub fn is_error(&self) -> bool {
        !self.success
    }

    /// Get error message if available
    pub fn get_error(&self) -> Option<&str> {
        self.error_message.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_capability_request_creation() {
        let request = CapabilityRequest::new("compute".to_string(), "process".to_string());

        assert_eq!(request.capability_type, "compute");
        assert_eq!(request.operation, "process");
        assert_eq!(request.timeout_ms, Some(30000));
        assert!(request.parameters.is_empty());
    }

    #[test]
    fn test_request_with_parameters() {
        let mut params = HashMap::new();
        params.insert("input".to_string(), json!("test data"));
        params.insert("iterations".to_string(), json!(10));

        let request =
            CapabilityRequest::with_parameters("ai".to_string(), "inference".to_string(), params);

        assert_eq!(
            request.get_string_parameter("input"),
            Some("test data".to_string())
        );
        assert_eq!(request.get_number_parameter("iterations"), Some(10.0));
    }

    #[test]
    fn test_request_parameter_methods() {
        let request = CapabilityRequest::new("test".to_string(), "test".to_string())
            .add_parameter("string_param".to_string(), json!("hello"))
            .add_parameter("number_param".to_string(), json!(42))
            .add_parameter("bool_param".to_string(), json!(true))
            .with_timeout(5000);

        assert_eq!(
            request.get_string_parameter("string_param"),
            Some("hello".to_string())
        );
        assert_eq!(request.get_number_parameter("number_param"), Some(42.0));
        assert_eq!(request.get_bool_parameter("bool_param"), Some(true));
        assert_eq!(request.timeout_ms, Some(5000));

        assert!(request.has_parameter("string_param"));
        assert!(!request.has_parameter("missing_param"));
    }

    #[test]
    fn test_validate_required_parameters() {
        let request = CapabilityRequest::new("test".to_string(), "test".to_string())
            .add_parameter("required1".to_string(), json!("value1"))
            .add_parameter("required2".to_string(), json!("value2"));

        // Should succeed with all required parameters present
        assert!(request
            .validate_required_parameters(&["required1", "required2"])
            .is_ok());

        // Should fail with missing parameter
        assert!(request
            .validate_required_parameters(&["required1", "missing"])
            .is_err());
    }

    #[test]
    fn test_capability_response_success() {
        let request_id = Uuid::new_v4();
        let data = json!({"result": "success", "value": 42});

        let response = CapabilityResponse::success(request_id, data.clone())
            .with_execution_time(100)
            .with_provider_id("test-provider".to_string())
            .with_metadata("version".to_string(), "1.0".to_string());

        assert!(response.is_success());
        assert!(!response.is_error());
        assert_eq!(response.request_id, request_id);
        assert_eq!(response.data, data);
        assert_eq!(response.execution_time_ms, Some(100));
        assert_eq!(response.provider_id, Some("test-provider".to_string()));
        assert!(response.get_error().is_none());
    }

    #[test]
    fn test_capability_response_error() {
        let request_id = Uuid::new_v4();
        let error_msg = "Operation failed".to_string();

        let response = CapabilityResponse::error(request_id, error_msg.clone());

        assert!(!response.is_success());
        assert!(response.is_error());
        assert_eq!(response.request_id, request_id);
        assert_eq!(response.get_error(), Some(error_msg.as_str()));
    }

    #[test]
    fn test_response_get_data() -> SongbirdResult<()> {
        #[derive(Deserialize, PartialEq, Debug)]
        struct TestData {
            result: String,
            value: i32,
        }

        let request_id = Uuid::new_v4();
        let data = json!({"result": "success", "value": 42});
        let response = CapabilityResponse::success(request_id, data);

        let parsed: TestData = response.get_data().map_err(|e| {
            Box::new(songbird_errors::SongbirdError::operation_error(format!(
                "Operation failed: {}",
                e
            ))) as Box<dyn std::error::Error>
        })?;
        assert_eq!(
            parsed,
            TestData {
                result: "success".to_string(),
                value: 42,
            }
        );
        Ok(SongbirdResponse::success(()))
    }
}
