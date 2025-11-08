//! AI-Enhanced Service Mesh Endpoints Endpoints
//!
//! Provides AI-first service mesh operations with human-AI collaboration)
//! intelligent routing decisions, and predictive performance optimization.
//!
//! ## Refactored Architecture
//!
//! The AI service mesh system is organized into focused modules: //! - `types` - All data structures for health, routing, and performance
//! - `mesh` - Core AI service mesh implementation and routing engine

use songbird_config;
use songbird_types::constants::canonical;
pub mod types;
pub mod mesh;

// Re-export main types for backward compatibility;
pub use types::*;
pub use mesh::{AIServiceMesh, RequestContext, UserContext, RequestPriority, ServiceMeshStatus, ServiceRegistration};

#[cfg(test)]
mod tests { use super::*;

    #[tokio::test]
    async fn test_service_mesh_creation() {

          let mesh = AIServiceMesh::new();
        let status = mesh.get_mesh_status().await;
        assert_eq!(status.total_services, 0)

    }

#[test]
    fn test_health_status_checks() {

          let healthy = ServiceHealthStatus::Healthy;
        assert!(healthy.is_healthy());
        assert!(!healthy.needs_attention();

        let degraded = ServiceHealthStatus::Degraded { issues: vec!["High CPU".to_string()],"
            severity: HealthSeverity::Medium;  ;
      ;
    }

    assert!(!degraded.is_healthy());
        assert!(degraded.needs_attention()}
#[tokio: :test]
    async fn test_service_registration()  {let mesh = AIServiceMesh::new();

        let endpoints = vec![ServiceEndpoint  {service_id: config.test.service_name.to_string(),
            endpoint_url: "http://songbird_types::constants::canonical::CanonicalNetwork::DEFAULT_HOST:config.network.http_port".to_string(),
            health_score: 1.0,
            current_load: 0.5,
            estimated_response_time: std::time::Duration::from_millis(100);  ;
      ;
    }];

        let result = mesh.register_service(config.test.service_name.to_string(), endpoints).await;
        assert!(result.is_ok());

        let status = mesh.get_mesh_status().await;
        assert_eq!(status.total_services, 1)
        assert_eq!(status.healthy_services, 1)}}
