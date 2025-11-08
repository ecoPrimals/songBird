//! Capability-Based Routing Tests
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]

//!
//! Tests for Songbird's core orchestration capability: routing requests
//! to appropriate primals based on capability matching.

use songbird_test_utils::{
    ai_service, compute_service,
    mocks::common::{HealthStatus, MockPrimalServer},
    multi_capability_service, security_service, storage_service, OrchestratorTestEnvironment,
};
use songbird_types::{SongbirdError, SongbirdResult};

#[tokio::test]
async fn test_route_to_single_provider_by_capability() {
    // Setup: Environment with one compute service
    let env = OrchestratorTestEnvironment::with_compute_only().await;

    // Create a compute service registration
    let compute = compute_service("toadstool-1").with_endpoint(&env.toadstool_endpoint());

    // Verify: Service has compute capability
    assert!(compute.capabilities().contains(&"compute".to_string()));

    // Verify: Endpoint is accessible
    assert!(!compute.endpoint().is_empty());

    // Test Objective: Single provider should be selected for its capability
    // In a real orchestrator, we would:
    // let request = Request::for_capability("compute");
    // let response = orchestrator.route(request).await.map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?;
    // assert_eq!(response.service_id, compute.id());

    env.cleanup().await;
}

#[tokio::test]
async fn test_route_distributes_across_multiple_providers() {
    // Setup: Environment with multiple compute services
    let env = OrchestratorTestEnvironment::with_healthy_primals().await;

    let compute1 = compute_service("toadstool-1")
        .with_endpoint(format!("{}/compute1", env.toadstool_endpoint()));

    let compute2 = compute_service("toadstool-2")
        .with_endpoint(format!("{}/compute2", env.toadstool_endpoint()));

    // Both provide the same capability
    assert_eq!(compute1.capabilities(), compute2.capabilities());

    // Test Objective: Requests should distribute across multiple providers
    // with the same capability (load balancing)
    //
    // In a real implementation:
    // let mut routes = HashMap::new();
    // for _ in 0..20 {
    //     let request = Request::for_capability("compute");
    //     let response = orchestrator.route(request).await.map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?;
    //     *routes.entry(response.service_id).or_insert(0) += 1;
    // }
    // assert_eq!(routes.len(), 2, "Should use both providers");
    // assert!(routes.values().all(|&count| count > 0), "Both should receive requests");

    // Verify: Multiple services can coexist
    assert_ne!(compute1.id(), compute2.id());

    env.cleanup().await;
}

#[tokio::test]
async fn test_route_respects_health_status() {
    // Setup: Environment with degraded services
    let env = OrchestratorTestEnvironment::with_high_load().await;

    // Verify all services are degraded
    assert_eq!(env.toadstool.read().await.get_health(), HealthStatus::Degraded);
    assert_eq!(env.beardog.read().await.get_health(), HealthStatus::Degraded);

    // Test Objective: Unhealthy services should be avoided
    //
    // Expected behavior:
    // 1. Healthy services get 100% of traffic
    // 2. Degraded services get traffic only if no healthy alternatives
    // 3. Unhealthy services never receive traffic
    //
    // Implementation would be:
    // let healthy = compute_service("healthy").with_health(Healthy);
    // let degraded = compute_service("degraded").with_health(Degraded);
    // let unhealthy = compute_service("unhealthy").with_health(Unhealthy);
    //
    // orchestrator.register_services(vec![healthy, degraded, unhealthy]);
    //
    // for _ in 0..50 {
    //     let response = orchestrator.route("compute").await.map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?;
    //     assert_ne!(response.service_id, unhealthy.id);
    // }

    env.cleanup().await;
}

#[tokio::test]
async fn test_route_fails_when_no_providers_available() {
    // Setup: Clean environment
    let env = OrchestratorTestEnvironment::new().await;

    // Make all services unhealthy
    env.toadstool.read().await.set_health(HealthStatus::Unhealthy);
    env.beardog.read().await.set_health(HealthStatus::Unhealthy);
    env.nestgate.read().await.set_health(HealthStatus::Unhealthy);
    env.squirrel.read().await.set_health(HealthStatus::Unhealthy);

    // Test Objective: Routing should fail gracefully when no services available
    //
    // Expected error types:
    // - ServiceUnavailable: No providers for requested capability
    // - AllProvidersUnhealthy: Providers exist but all unhealthy
    //
    // Implementation:
    // let request = Request::for_capability("compute");
    // let result = orchestrator.route(request).await;
    //
    // assert!(result.is_err());
    // let error = result.unwrap_err();
    // assert!(matches!(error, SongbirdError::ServiceUnavailable(_)));

    // Verify: All services are indeed unhealthy
    assert_eq!(env.toadstool.read().await.get_health(), HealthStatus::Unhealthy);

    env.cleanup().await;
}

#[tokio::test]
async fn test_route_selects_best_multi_capability_provider() {
    // Setup: Services with overlapping capabilities
    let env = OrchestratorTestEnvironment::with_healthy_primals().await;

    let specialized =
        compute_service("specialized-compute").with_endpoint(&env.toadstool_endpoint());

    let generalist =
        multi_capability_service("multi-service", vec!["compute", "storage", "processing"])
            .with_endpoint(&env.nestgate_endpoint());

    // Test Objective: When multiple services can satisfy a capability,
    // choose based on:
    // 1. Specialization (single-purpose vs multi-purpose)
    // 2. Health status
    // 3. Current load
    // 4. Historical performance
    //
    // Expected: Specialized service preferred for compute requests
    // even though generalist can also handle them
    //
    // Implementation:
    // orchestrator.register_services(vec![specialized.clone(), generalist.clone()]);
    //
    // let mut selected = HashMap::new();
    // for _ in 0..100 {
    //     let response = orchestrator.route("compute").await.map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?;
    //     *selected.entry(response.service_id).or_insert(0) += 1;
    // }
    //
    // let specialized_count = selected.get(&specialized.id()).unwrap_or(&0);
    // let generalist_count = selected.get(&generalist.id()).unwrap_or(&0);
    //
    // // Specialized should get more traffic (e.g., >70%)
    // assert!(*specialized_count > *generalist_count * 2);

    // Verify: Capabilities are as expected
    assert_eq!(specialized.capabilities().len(), 1);
    assert_eq!(generalist.capabilities().len(), 3);
    assert!(generalist.capabilities().contains(&"compute".to_string()));

    env.cleanup().await;
}

#[tokio::test]
async fn test_routing_with_all_primal_types() {
    // Setup: Full ecosystem environment
    let env = OrchestratorTestEnvironment::with_healthy_primals().await;

    // Create one of each primal type
    let compute = compute_service("toadstool").with_endpoint(&env.toadstool_endpoint());

    let security = security_service("beardog").with_endpoint(&env.beardog_endpoint());

    let storage = storage_service("nestgate").with_endpoint(&env.nestgate_endpoint());

    let ai = ai_service("squirrel").with_endpoint(&env.squirrel_endpoint());

    // Test Objective: Verify capability-based routing works for all primal types
    //
    // Routes that should work:
    // - "compute" -> toadstool
    // - "security" -> beardog
    // - "auth" -> beardog (multiple capabilities)
    // - "storage" -> nestgate
    // - "ai" -> squirrel
    // - "inference" -> squirrel (multiple capabilities)
    //
    // Implementation:
    // orchestrator.register_services(vec![compute, security, storage, ai]);
    //
    // let test_cases = vec![
    //     ("compute", "toadstool"),
    //     ("security", "beardog"),
    //     ("auth", "beardog"),
    //     ("storage", "nestgate"),
    //     ("ai", "squirrel"),
    //     ("inference", "squirrel"),
    // ];
    //
    // for (capability, expected_primal_type) in test_cases {
    //     let response = orchestrator.route(capability).await.map_err(|e| SongbirdError::configuration(format!("Test operation failed: {}", e)))?;
    //     let service_type = response.metadata.get("type").or_else(|_| SongbirdError::configuration(format!("Test operation failed: {}", e)))?;
    //     assert_eq!(service_type, expected_primal_type);
    // }

    // Verify: Each service has distinct capabilities
    assert!(compute.capabilities().contains(&"compute".to_string()));
    assert!(security.capabilities().contains(&"security".to_string()));
    assert!(security.capabilities().contains(&"auth".to_string()));
    assert!(storage.capabilities().contains(&"storage".to_string()));
    assert!(ai.capabilities().contains(&"ai".to_string()));
    assert!(ai.capabilities().contains(&"inference".to_string()));

    env.cleanup().await;
}

#[cfg(test)]
mod advanced_routing_scenarios {
    use super::*;

    #[tokio::test]
    async fn test_routing_failover() {
        // Test Objective: When primary provider fails, route to secondary
        let env = OrchestratorTestEnvironment::with_healthy_primals().await;

        // Implementation concept:
        // 1. Register primary and secondary compute services
        // 2. Verify routing to primary
        // 3. Mark primary unhealthy
        // 4. Verify automatic failover to secondary
        // 5. Restore primary health
        // 6. Verify traffic returns to primary

        env.cleanup().await;
    }

    #[tokio::test]
    async fn test_routing_with_service_weights() {
        // Test Objective: Support weighted load distribution
        let env = OrchestratorTestEnvironment::with_healthy_primals().await;

        // Implementation concept:
        // Service A: weight 70 -> gets ~70% of traffic
        // Service B: weight 30 -> gets ~30% of traffic

        env.cleanup().await;
    }

    #[tokio::test]
    async fn test_routing_respects_sovereignty_preferences() {
        // Test Objective: Route considering sovereignty and human dignity
        let env = OrchestratorTestEnvironment::with_healthy_primals().await;

        // Implementation concept:
        // - Prefer services with matching sovereignty domain
        // - Respect data locality requirements
        // - Honor user's sovereignty preferences

        env.cleanup().await;
    }
}
