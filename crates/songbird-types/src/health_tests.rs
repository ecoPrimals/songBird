//! Comprehensive tests for health types
//!
//! Tests for CanonicalHealthStatus and CanonicalHealthCheck

#[cfg(test)]
mod tests {
    use super::super::health::{CanonicalHealthCheck, CanonicalHealthStatus};
    use std::collections::HashMap;

    // ============================================================================
    // CanonicalHealthStatus Tests
    // ============================================================================

    #[test]
    fn test_health_status_default() {
        let status = CanonicalHealthStatus::default();
        assert_eq!(status, CanonicalHealthStatus::Unknown);
    }

    #[test]
    fn test_health_status_variants() {
        // Test all variants exist and are distinct
        let healthy = CanonicalHealthStatus::Healthy;
        let degraded = CanonicalHealthStatus::Degraded;
        let unhealthy = CanonicalHealthStatus::Unhealthy;
        let unknown = CanonicalHealthStatus::Unknown;

        assert_ne!(healthy, degraded);
        assert_ne!(healthy, unhealthy);
        assert_ne!(healthy, unknown);
        assert_ne!(degraded, unhealthy);
        assert_ne!(degraded, unknown);
        assert_ne!(unhealthy, unknown);
    }

    #[test]
    fn test_health_status_display() {
        assert_eq!(format!("{}", CanonicalHealthStatus::Healthy), "Healthy");
        assert_eq!(format!("{}", CanonicalHealthStatus::Degraded), "Degraded");
        assert_eq!(format!("{}", CanonicalHealthStatus::Unhealthy), "Unhealthy");
        assert_eq!(format!("{}", CanonicalHealthStatus::Unknown), "Unknown");
    }

    #[test]
    fn test_health_status_clone() {
        let status = CanonicalHealthStatus::Healthy;
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }

    #[test]
    fn test_health_status_debug() {
        let status = CanonicalHealthStatus::Healthy;
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("Healthy"));
    }

    #[test]
    fn test_health_status_serialization() {
        let status = CanonicalHealthStatus::Healthy;
        let json = serde_json::to_string(&status).expect("Should serialize");
        assert!(json.contains("Healthy"));
    }

    #[test]
    fn test_health_status_deserialization() {
        let json = r#""Healthy""#;
        let status: CanonicalHealthStatus = serde_json::from_str(json).expect("Should deserialize");
        assert_eq!(status, CanonicalHealthStatus::Healthy);
    }

    // ============================================================================
    // CanonicalHealthCheck Tests
    // ============================================================================

    #[test]
    fn test_health_check_default() {
        let check = CanonicalHealthCheck::default();
        assert_eq!(check.status, CanonicalHealthStatus::Unknown);
        assert_eq!(check.message, None);
        assert!(check.metrics.is_empty());
        assert!(check.components.is_empty());
    }

    #[test]
    fn test_health_check_healthy() {
        let check = CanonicalHealthCheck::healthy();
        assert_eq!(check.status, CanonicalHealthStatus::Healthy);
        assert!(check.message.is_some());
        assert_eq!(check.message.unwrap(), "All systems operational");
    }

    #[test]
    fn test_health_check_degraded() {
        let check = CanonicalHealthCheck::degraded("Service slow");
        assert_eq!(check.status, CanonicalHealthStatus::Degraded);
        assert_eq!(check.message, Some("Service slow".to_string()));
    }

    #[test]
    fn test_health_check_unhealthy() {
        let check = CanonicalHealthCheck::unhealthy("Database down");
        assert_eq!(check.status, CanonicalHealthStatus::Unhealthy);
        assert_eq!(check.message, Some("Database down".to_string()));
    }

    #[test]
    fn test_health_check_with_metrics() {
        let mut metrics = HashMap::new();
        metrics.insert("cpu_usage".to_string(), 45.5);
        metrics.insert("memory_usage".to_string(), 78.2);

        let mut check = CanonicalHealthCheck::healthy();
        check.metrics = metrics;

        assert_eq!(check.metrics.len(), 2);
        assert_eq!(check.metrics.get("cpu_usage"), Some(&45.5));
        assert_eq!(check.metrics.get("memory_usage"), Some(&78.2));
    }

    #[test]
    fn test_health_check_with_components() {
        let mut components = HashMap::new();
        components.insert("database".to_string(), CanonicalHealthStatus::Healthy);
        components.insert("cache".to_string(), CanonicalHealthStatus::Degraded);
        components.insert("queue".to_string(), CanonicalHealthStatus::Unhealthy);

        let mut check = CanonicalHealthCheck::healthy();
        check.components = components;

        assert_eq!(check.components.len(), 3);
        assert_eq!(check.components.get("database"), Some(&CanonicalHealthStatus::Healthy));
        assert_eq!(check.components.get("cache"), Some(&CanonicalHealthStatus::Degraded));
        assert_eq!(check.components.get("queue"), Some(&CanonicalHealthStatus::Unhealthy));
    }

    #[test]
    fn test_health_check_clone() {
        let check = CanonicalHealthCheck::healthy();
        let cloned = check.clone();
        assert_eq!(check.status, cloned.status);
        assert_eq!(check.message, cloned.message);
    }

    #[test]
    fn test_health_check_debug() {
        let check = CanonicalHealthCheck::healthy();
        let debug_str = format!("{:?}", check);
        assert!(debug_str.contains("Healthy"));
    }

    #[test]
    fn test_health_check_serialization() {
        let check = CanonicalHealthCheck::healthy();
        let json = serde_json::to_string(&check).expect("Should serialize");
        assert!(json.contains("Healthy"));
        assert!(json.contains("All systems operational"));
    }

    #[test]
    fn test_health_check_deserialization() {
        let json = r#"{
            "status": "Healthy",
            "message": "Test message",
            "metrics": {},
            "components": {}
        }"#;
        let check: CanonicalHealthCheck = serde_json::from_str(json).expect("Should deserialize");
        assert_eq!(check.status, CanonicalHealthStatus::Healthy);
        assert_eq!(check.message, Some("Test message".to_string()));
    }

    #[test]
    fn test_health_check_complex_scenario() {
        // Create a complex health check with all fields populated
        let mut metrics = HashMap::new();
        metrics.insert("requests_per_sec".to_string(), 1250.5);
        metrics.insert("error_rate".to_string(), 0.02);
        metrics.insert("latency_p99".to_string(), 145.3);

        let mut components = HashMap::new();
        components.insert("api".to_string(), CanonicalHealthStatus::Healthy);
        components.insert("worker".to_string(), CanonicalHealthStatus::Healthy);
        components.insert("cache".to_string(), CanonicalHealthStatus::Degraded);

        let mut check = CanonicalHealthCheck::healthy();
        check.metrics = metrics;
        check.components = components;

        // Serialize and deserialize
        let json = serde_json::to_string(&check).expect("Should serialize");
        let deserialized: CanonicalHealthCheck =
            serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(deserialized.status, CanonicalHealthStatus::Healthy);
        assert_eq!(deserialized.metrics.len(), 3);
        assert_eq!(deserialized.components.len(), 3);
    }

    #[test]
    fn test_health_status_equality() {
        let status1 = CanonicalHealthStatus::Healthy;
        let status2 = CanonicalHealthStatus::Healthy;
        let status3 = CanonicalHealthStatus::Degraded;

        assert_eq!(status1, status2);
        assert_ne!(status1, status3);
    }

    #[test]
    fn test_health_check_empty_message() {
        let check = CanonicalHealthCheck::degraded("");
        assert_eq!(check.message, Some("".to_string()));
    }

    #[test]
    fn test_health_check_long_message() {
        let long_message = "A".repeat(1000);
        let check = CanonicalHealthCheck::unhealthy(long_message.clone());
        assert_eq!(check.message, Some(long_message));
    }

    #[test]
    fn test_health_check_special_characters_in_message() {
        let check = CanonicalHealthCheck::degraded("Error: 🔥 Database connection failed!");
        assert!(check.message.unwrap().contains("🔥"));
    }

    #[test]
    fn test_health_check_metrics_empty() {
        let check = CanonicalHealthCheck::healthy();
        assert!(check.metrics.is_empty());
        assert_eq!(check.metrics.len(), 0);
    }

    #[test]
    fn test_health_check_components_empty() {
        let check = CanonicalHealthCheck::healthy();
        assert!(check.components.is_empty());
        assert_eq!(check.components.len(), 0);
    }

    #[test]
    fn test_health_status_copy_trait() {
        let status = CanonicalHealthStatus::Healthy;
        let copied = status; // Copy happens here
        assert_eq!(status, copied); // Original still usable
    }
}
