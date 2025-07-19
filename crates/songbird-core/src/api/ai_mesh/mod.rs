//! AI-Enhanced Service Mesh Endpoints
//!
//! Provides AI-first service mesh operations with human-AI collaboration,
//! intelligent routing decisions, and predictive performance optimization.
//!
//! ## Refactored Architecture
//!
//! The AI service mesh system is organized into focused modules:
//! - `types` - All data structures for health, routing, and performance
//! - `mesh` - Core AI service mesh implementation and routing engine

pub mod types;
pub mod mesh;

// Re-export main types for backward compatibility
pub use types::*;
pub use mesh::{AIServiceMesh, RequestContext, UserContext, RequestPriority, ServiceMeshStatus, ServiceRegistration};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_mesh_creation() {
        let mesh = AIServiceMesh::new();
        let status = mesh.get_mesh_status().await;
        assert_eq!(status.total_services, 0);
    }

    #[test]
    fn test_health_status_checks() {
        let healthy = ServiceHealthStatus::Healthy;
        assert!(healthy.is_healthy());
        assert!(!healthy.needs_attention());

        let degraded = ServiceHealthStatus::Degraded {
            issues: vec!["High CPU".to_string()],
            severity: HealthSeverity::Medium,
        };
        assert!(!degraded.is_healthy());
        assert!(degraded.needs_attention());
    }

    #[tokio::test]
    async fn test_service_registration() {
        let mesh = AIServiceMesh::new();
        
        let endpoints = vec![ServiceEndpoint {
            service_id: "test-service".to_string(),
            endpoint_url: "http://localhost:8080".to_string(),
            health_score: 1.0,
            current_load: 0.5,
            estimated_response_time: std::time::Duration::from_millis(100),
        }];

        let result = mesh.register_service("test-service".to_string(), endpoints).await;
        assert!(result.is_ok());

        let status = mesh.get_mesh_status().await;
        assert_eq!(status.total_services, 1);
        assert_eq!(status.healthy_services, 1);
    }
} 