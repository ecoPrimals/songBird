//! Operation result types for different provider types

use serde::{Deserialize, Serialize};

/// System metrics from compute providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub disk_usage_percent: f64,
    pub network_io_bytes_per_sec: u64,
    pub active_processes: u32,
    pub uptime_seconds: u64,
}

/// Storage operation results from storage providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageOperationResult {
    pub operation_type: String,
    pub key: String,
    pub size_bytes: Option<u64>,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Security operation results from security providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityOperationResult {
    pub operation_type: String,
    pub success: bool,
    pub encrypted_data: Option<String>,
    pub decrypted_data: Option<String>,
    pub error_message: Option<String>,
}

/// AI operation results from AI providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIOperationResult {
    pub operation_type: String,
    pub input_data: String,
    pub output_data: Option<serde_json::Value>,
    pub confidence_score: Option<f64>,
    pub processing_time_ms: Option<u64>,
    pub success: bool,
    pub error_message: Option<String>,
}

impl SystemMetrics {
    /// Create new system metrics
    pub fn new(
        cpu_usage: f64,
        memory_usage: f64,
        disk_usage: f64,
        network_io: u64,
        processes: u32,
        uptime: u64,
    ) -> Self {
        Self {
            cpu_usage_percent: cpu_usage,
            memory_usage_percent: memory_usage,
            disk_usage_percent: disk_usage,
            network_io_bytes_per_sec: network_io,
            active_processes: processes,
            uptime_seconds: uptime,
        }
    }

    /// Check if system is under high load
    pub fn is_high_load(&self) -> bool {
        self.cpu_usage_percent > 80.0 || self.memory_usage_percent > 90.0
    }

    /// Get overall system health score (0.0 to 1.0)
    pub fn health_score(&self) -> f64 {
        let cpu_score = (100.0 - self.cpu_usage_percent) / 100.0;
        let memory_score = (100.0 - self.memory_usage_percent) / 100.0;
        let disk_score = (100.0 - self.disk_usage_percent) / 100.0;

        (cpu_score + memory_score + disk_score) / 3.0
    }
}

impl StorageOperationResult {
    /// Create a successful storage operation result
    pub fn success(operation_type: String, key: String, size_bytes: Option<u64>) -> Self {
        Self {
            operation_type,
            key,
            size_bytes,
            success: true,
            error_message: None,
        }
    }

    /// Create a failed storage operation result
    pub fn failure(operation_type: String, key: String, error_message: String) -> Self {
        Self {
            operation_type,
            key,
            size_bytes: None,
            success: false,
            error_message: Some(error_message),
        }
    }

    /// Check if operation was successful
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Get error message if available
    pub fn error(&self) -> Option<&str> {
        self.error_message.as_deref()
    }
}

impl SecurityOperationResult {
    /// Create a successful encryption result
    pub fn encryption_success(encrypted_data: String) -> Self {
        Self {
            operation_type: "encrypt".to_string(),
            success: true,
            encrypted_data: Some(encrypted_data),
            decrypted_data: None,
            error_message: None,
        }
    }

    /// Create a successful decryption result
    pub fn decryption_success(decrypted_data: String) -> Self {
        Self {
            operation_type: "decrypt".to_string(),
            success: true,
            encrypted_data: None,
            decrypted_data: Some(decrypted_data),
            error_message: None,
        }
    }

    /// Create a failed security operation result
    pub fn failure(operation_type: String, error_message: String) -> Self {
        Self {
            operation_type,
            success: false,
            encrypted_data: None,
            decrypted_data: None,
            error_message: Some(error_message),
        }
    }

    /// Check if operation was successful
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Get error message if available
    pub fn error(&self) -> Option<&str> {
        self.error_message.as_deref()
    }
}

impl AIOperationResult {
    /// Create a successful AI operation result
    pub fn success(
        operation_type: String,
        input_data: String,
        output_data: serde_json::Value,
        confidence_score: Option<f64>,
        processing_time_ms: Option<u64>,
    ) -> Self {
        Self {
            operation_type,
            input_data,
            output_data: Some(output_data),
            confidence_score,
            processing_time_ms,
            success: true,
            error_message: None,
        }
    }

    /// Create a failed AI operation result
    pub fn failure(operation_type: String, input_data: String, error_message: String) -> Self {
        Self {
            operation_type,
            input_data,
            output_data: None,
            confidence_score: None,
            processing_time_ms: None,
            success: false,
            error_message: Some(error_message),
        }
    }

    /// Check if operation was successful
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Check if result has high confidence
    pub fn is_high_confidence(&self) -> bool {
        self.confidence_score.is_some_and(|score| score > 0.8)
    }

    /// Get error message if available
    pub fn error(&self) -> Option<&str> {
        self.error_message.as_deref()
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            memory_usage_percent: 0.0,
            disk_usage_percent: 0.0,
            network_io_bytes_per_sec: 0,
            active_processes: 0,
            uptime_seconds: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_system_metrics() {
        let metrics = SystemMetrics::new(50.0, 60.0, 70.0, 1000, 25, 3600);

        assert_eq!(metrics.cpu_usage_percent, 50.0);
        assert_eq!(metrics.memory_usage_percent, 60.0);
        assert_eq!(metrics.disk_usage_percent, 70.0);
        assert!(!metrics.is_high_load());

        let health = metrics.health_score();
        assert!(health > 0.0 && health <= 1.0);
    }

    #[test]
    fn test_system_high_load() {
        let high_cpu = SystemMetrics::new(85.0, 50.0, 50.0, 1000, 25, 3600);
        assert!(high_cpu.is_high_load());

        let high_memory = SystemMetrics::new(50.0, 95.0, 50.0, 1000, 25, 3600);
        assert!(high_memory.is_high_load());

        let normal = SystemMetrics::new(50.0, 60.0, 70.0, 1000, 25, 3600);
        assert!(!normal.is_high_load());
    }

    #[test]
    fn test_storage_operation_result() {
        let success = StorageOperationResult::success(
            "store".to_string(),
            "test-key".to_string(),
            Some(1024),
        );

        assert!(success.is_success());
        assert_eq!(success.operation_type, "store");
        assert_eq!(success.key, "test-key");
        assert_eq!(success.size_bytes, Some(1024));
        assert!(success.error().is_none());

        let failure = StorageOperationResult::failure(
            "retrieve".to_string(),
            "missing-key".to_string(),
            "Key not found".to_string(),
        );

        assert!(!failure.is_success());
        assert_eq!(failure.error(), Some("Key not found"));
    }

    #[test]
    fn test_security_operation_result() {
        let encryption = SecurityOperationResult::encryption_success("encrypted123".to_string());
        assert!(encryption.is_success());
        assert_eq!(encryption.operation_type, "encrypt");
        assert_eq!(encryption.encrypted_data, Some("encrypted123".to_string()));

        let decryption = SecurityOperationResult::decryption_success("decrypted456".to_string());
        assert!(decryption.is_success());
        assert_eq!(decryption.operation_type, "decrypt");
        assert_eq!(decryption.decrypted_data, Some("decrypted456".to_string()));

        let failure =
            SecurityOperationResult::failure("encrypt".to_string(), "Invalid key".to_string());
        assert!(!failure.is_success());
        assert_eq!(failure.error(), Some("Invalid key"));
    }

    #[test]
    fn test_ai_operation_result() {
        let success = AIOperationResult::success(
            "inference".to_string(),
            "input text".to_string(),
            json!({"result": "processed"}),
            Some(0.95),
            Some(150),
        );

        assert!(success.is_success());
        assert!(success.is_high_confidence());
        assert_eq!(success.operation_type, "inference");
        assert_eq!(success.input_data, "input text");
        assert_eq!(success.confidence_score, Some(0.95));
        assert_eq!(success.processing_time_ms, Some(150));

        let failure = AIOperationResult::failure(
            "inference".to_string(),
            "invalid input".to_string(),
            "Model error".to_string(),
        );

        assert!(!failure.is_success());
        assert!(!failure.is_high_confidence());
        assert_eq!(failure.error(), Some("Model error"));
    }
}
