//! Universal Service Registration Manager
//!
//! Core manager implementation for universal service registration

use crate::api::ai_first_response::{
    AIErrorCategory, AIFirstError, AIFirstResponse, AIResponseMetadata, ActionPriority,
    HumanInteractionContext, SuggestedAction,
};
use crate::api::universal_service_registration::ai_components::*;
use crate::api::universal_service_registration::types::*;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

/// Universal Service Registration Manager
pub struct UniversalServiceRegistrationManager {
    // Internal service registry
    registered_services: HashMap<String, RegisteredServiceInfo>,

    // AI prediction engine
    performance_predictor: PerformancePredictor,

    // Human interaction manager
    human_interaction_manager: HumanInteractionManager,

    // Service mesh integration
    service_mesh_integrator: ServiceMeshIntegrator,
}

impl Default for UniversalServiceRegistrationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl UniversalServiceRegistrationManager {
    pub fn new() -> Self {
        Self {
            registered_services: HashMap::new(),
            performance_predictor: PerformancePredictor::new(),
            human_interaction_manager: HumanInteractionManager::new(),
            service_mesh_integrator: ServiceMeshIntegrator::new(),
        }
    }

    /// Register a service with AI-first response format
    pub async fn register_service(
        &mut self,
        request: UniversalServiceRegistrationRequest,
        human_context: Option<HumanInteractionContext>,
    ) -> AIFirstResponse<ServiceRegistrationData> {
        let request_id = Uuid::new_v4();
        let start_time = std::time::Instant::now();

        // Validate service request
        if let Err(validation_error) = self.validate_service_request(&request).await {
            return AIFirstResponse::error(
                ServiceRegistrationData {
                    service_id: request.service_id.clone(),
                    registered_at: Utc::now(),
                    routing_info: ServiceMeshRoutingInfo {
                        routing_rules: vec![],
                        traffic_percentage: 0.0,
                        circuit_breaker_config: CircuitBreakerConfig {
                            failure_threshold_percentage: 50.0,
                            minimum_request_threshold: 10,
                            sleep_window_seconds: 30,
                            request_volume_threshold: 20,
                        },
                        priority: ServicePriority::Low,
                    },
                    load_balancing_pool: None,
                    monitoring_config: MonitoringConfiguration {
                        metrics_interval_seconds: 60,
                        health_check_frequency_seconds: 30,
                        performance_baseline: PerformanceBaseline {
                            expected_response_time_ms: 100.0,
                            expected_throughput_rps: 100.0,
                            expected_error_rate_percentage: 1.0,
                            expected_cpu_utilization_percentage: 50.0,
                        },
                        alert_thresholds: AlertThresholds {
                            response_time_threshold_ms: 1000.0,
                            error_rate_threshold_percentage: 5.0,
                            cpu_utilization_threshold_percentage: 80.0,
                            memory_utilization_threshold_percentage: 80.0,
                        },
                    },
                    performance_predictions: PerformancePredictions {
                        predicted_latency_ms: 0.0,
                        predicted_max_throughput_rps: 0.0,
                        scaling_predictions: ScalingPredictions {
                            horizontal_scaling_efficiency: 0.0,
                            recommended_min_instances: 1,
                            recommended_max_instances: 1,
                            auto_scaling_triggers: vec![],
                        },
                        resource_efficiency_score: 0.0,
                        predicted_reliability_percentage: 0.0,
                    },
                },
                validation_error,
                request_id,
                start_time.elapsed().as_millis() as u64,
            );
        }

        // Check if human approval is required
        let needs_human_approval = self
            .assess_human_approval_need(&request, &human_context)
            .await;

        if needs_human_approval {
            // Notify human interaction manager
            let _ = self
                .human_interaction_manager
                .notify_registration_pending(&request.service_id)
                .await;

            // Return response indicating human approval needed
            let mut metadata = AIResponseMetadata::default();
            metadata.dependencies.push("human_approval".to_string());

            return AIFirstResponse::success(
                ServiceRegistrationData {
                    service_id: request.service_id.clone(),
                    registered_at: Utc::now(),
                    routing_info: ServiceMeshRoutingInfo {
                        routing_rules: vec![],
                        traffic_percentage: 0.0,
                        circuit_breaker_config: CircuitBreakerConfig {
                            failure_threshold_percentage: 50.0,
                            minimum_request_threshold: 10,
                            sleep_window_seconds: 30,
                            request_volume_threshold: 20,
                        },
                        priority: ServicePriority::Normal,
                    },
                    load_balancing_pool: None,
                    monitoring_config: MonitoringConfiguration {
                        metrics_interval_seconds: 60,
                        health_check_frequency_seconds: 30,
                        performance_baseline: PerformanceBaseline {
                            expected_response_time_ms: 100.0,
                            expected_throughput_rps: 100.0,
                            expected_error_rate_percentage: 1.0,
                            expected_cpu_utilization_percentage: 50.0,
                        },
                        alert_thresholds: AlertThresholds {
                            response_time_threshold_ms: 1000.0,
                            error_rate_threshold_percentage: 5.0,
                            cpu_utilization_threshold_percentage: 80.0,
                            memory_utilization_threshold_percentage: 80.0,
                        },
                    },
                    performance_predictions: PerformancePredictions {
                        predicted_latency_ms: 100.0,
                        predicted_max_throughput_rps: 1000.0,
                        scaling_predictions: ScalingPredictions {
                            horizontal_scaling_efficiency: 0.8,
                            recommended_min_instances: 1,
                            recommended_max_instances: 10,
                            auto_scaling_triggers: vec![],
                        },
                        resource_efficiency_score: 0.7,
                        predicted_reliability_percentage: 99.0,
                    },
                },
                request_id,
                start_time.elapsed().as_millis() as u64,
                0.5, // Lower confidence due to pending human approval
            )
            .with_ai_metadata(metadata)
            .with_suggested_actions(vec![SuggestedAction {
                action_type: "request_human_approval".to_string(),
                parameters: {
                    let mut params = HashMap::new();
                    params.insert(
                        "service_id".to_string(),
                        serde_json::Value::String(request.service_id.clone()),
                    );
                    params.insert(
                        "reason".to_string(),
                        serde_json::Value::String(
                            "New service registration requires approval".to_string(),
                        ),
                    );
                    params
                },
                priority: ActionPriority::High,
                expected_outcome: "Human approves or rejects service registration".to_string(),
                confidence: 0.9,
                requires_human_approval: true,
                estimated_execution_time: Some(std::time::Duration::from_secs(300)),
            }]);
        }

        // Proceed with automatic registration
        match self.perform_service_registration(request).await {
            Ok(registration_data) => {
                let mut metadata = AIResponseMetadata::default();
                metadata.performance.latency_ms = start_time.elapsed().as_millis() as f64;
                metadata.dependencies.push("service_mesh".to_string());
                metadata.dependencies.push("load_balancer".to_string());

                AIFirstResponse::success(
                    registration_data,
                    request_id,
                    start_time.elapsed().as_millis() as u64,
                    0.95, // High confidence for successful registration
                )
                .with_ai_metadata(metadata)
                .with_suggested_actions(vec![SuggestedAction {
                    action_type: "verify_service_health".to_string(),
                    parameters: HashMap::new(),
                    priority: ActionPriority::Medium,
                    expected_outcome: "Service health verified".to_string(),
                    confidence: 0.8,
                    requires_human_approval: false,
                    estimated_execution_time: Some(std::time::Duration::from_secs(30)),
                }])
            }
            Err(registration_error) => AIFirstResponse::error(
                ServiceRegistrationData {
                    service_id: "failed".to_string(),
                    registered_at: Utc::now(),
                    routing_info: ServiceMeshRoutingInfo {
                        routing_rules: vec![],
                        traffic_percentage: 0.0,
                        circuit_breaker_config: CircuitBreakerConfig {
                            failure_threshold_percentage: 50.0,
                            minimum_request_threshold: 10,
                            sleep_window_seconds: 30,
                            request_volume_threshold: 20,
                        },
                        priority: ServicePriority::Low,
                    },
                    load_balancing_pool: None,
                    monitoring_config: MonitoringConfiguration {
                        metrics_interval_seconds: 60,
                        health_check_frequency_seconds: 30,
                        performance_baseline: PerformanceBaseline {
                            expected_response_time_ms: 0.0,
                            expected_throughput_rps: 0.0,
                            expected_error_rate_percentage: 100.0,
                            expected_cpu_utilization_percentage: 0.0,
                        },
                        alert_thresholds: AlertThresholds {
                            response_time_threshold_ms: 1000.0,
                            error_rate_threshold_percentage: 5.0,
                            cpu_utilization_threshold_percentage: 80.0,
                            memory_utilization_threshold_percentage: 80.0,
                        },
                    },
                    performance_predictions: PerformancePredictions {
                        predicted_latency_ms: 0.0,
                        predicted_max_throughput_rps: 0.0,
                        scaling_predictions: ScalingPredictions {
                            horizontal_scaling_efficiency: 0.0,
                            recommended_min_instances: 0,
                            recommended_max_instances: 0,
                            auto_scaling_triggers: vec![],
                        },
                        resource_efficiency_score: 0.0,
                        predicted_reliability_percentage: 0.0,
                    },
                },
                registration_error,
                request_id,
                start_time.elapsed().as_millis() as u64,
            ),
        }
    }

    /// Get service information
    pub async fn get_service(&self, service_id: &str) -> Option<&RegisteredServiceInfo> {
        self.registered_services.get(service_id)
    }

    /// List all registered services
    pub async fn list_services(&self) -> Vec<String> {
        self.registered_services.keys().cloned().collect()
    }

    /// Deregister a service
    pub async fn deregister_service(&mut self, service_id: &str) -> Result<(), String> {
        if self.registered_services.remove(service_id).is_some() {
            let _ = self
                .service_mesh_integrator
                .remove_routing(service_id)
                .await;
            Ok(())
        } else {
            Err(format!("Service '{service_id}' not found"))
        }
    }

    /// Validate service registration request
    async fn validate_service_request(
        &self,
        request: &UniversalServiceRegistrationRequest,
    ) -> Result<(), AIFirstError> {
        // Validate service ID format
        if request.service_id.is_empty() {
            return Err(AIFirstError {
                code: "INVALID_SERVICE_ID".to_string(),
                message: "Service ID cannot be empty".to_string(),
                category: AIErrorCategory::ConfigurationIssue,
                retry_strategy: crate::api::ai_first_response::RetryStrategy {
                    should_retry: false,
                    delay_ms: 0,
                    max_attempts: 0,
                    backoff_strategy: crate::api::ai_first_response::BackoffType::Linear,
                    retry_conditions: vec![],
                    success_probability: 0.0,
                },
                automation_hints: vec!["Provide a valid service ID".to_string()],
                severity: crate::api::ai_first_response::ErrorSeverity::High,
                requires_human_intervention: true,
                context: HashMap::new(),
            });
        }

        // Validate primary endpoint
        if request.endpoints.primary.is_empty() {
            return Err(AIFirstError {
                code: "INVALID_PRIMARY_ENDPOINT".to_string(),
                message: "Primary endpoint cannot be empty".to_string(),
                category: AIErrorCategory::ConfigurationIssue,
                retry_strategy: crate::api::ai_first_response::RetryStrategy {
                    should_retry: false,
                    delay_ms: 0,
                    max_attempts: 0,
                    backoff_strategy: crate::api::ai_first_response::BackoffType::Linear,
                    retry_conditions: vec![],
                    success_probability: 0.0,
                },
                automation_hints: vec!["Provide a valid primary endpoint URL".to_string()],
                severity: crate::api::ai_first_response::ErrorSeverity::High,
                requires_human_intervention: true,
                context: HashMap::new(),
            });
        }

        // Check for duplicate service ID
        if self.registered_services.contains_key(&request.service_id) {
            return Err(AIFirstError {
                code: "DUPLICATE_SERVICE_ID".to_string(),
                message: format!("Service ID '{}' is already registered", request.service_id),
                category: AIErrorCategory::ConfigurationIssue,
                retry_strategy: crate::api::ai_first_response::RetryStrategy {
                    should_retry: false,
                    delay_ms: 0,
                    max_attempts: 0,
                    backoff_strategy: crate::api::ai_first_response::BackoffType::Linear,
                    retry_conditions: vec!["use_different_service_id".to_string()],
                    success_probability: 1.0,
                },
                automation_hints: vec![
                    "Use a different service ID".to_string(),
                    "Consider versioning the service ID".to_string(),
                ],
                severity: crate::api::ai_first_response::ErrorSeverity::Medium,
                requires_human_intervention: false,
                context: {
                    let mut ctx = HashMap::new();
                    ctx.insert(
                        "existing_service_id".to_string(),
                        serde_json::Value::String(request.service_id.clone()),
                    );
                    ctx
                },
            });
        }

        Ok(())
    }

    /// Assess whether human approval is needed for service registration
    async fn assess_human_approval_need(
        &self,
        request: &UniversalServiceRegistrationRequest,
        human_context: &Option<HumanInteractionContext>,
    ) -> bool {
        // Check if human interaction preferences require approval
        if let Some(prefs) = &request.human_interaction_preferences {
            if prefs.registration_approval_required {
                return true;
            }
        }

        // Check human context approval requirements
        if let Some(context) = human_context {
            if context.approval_required {
                return true;
            }
        }

        // Check for unknown primal types that might need approval
        if request.primal_type.contains("unknown")
            || request.primal_type.contains("custom")
            || request.primal_type.contains("community")
        {
            return true;
        }

        false
    }

    /// Perform the actual service registration
    async fn perform_service_registration(
        &mut self,
        request: UniversalServiceRegistrationRequest,
    ) -> Result<ServiceRegistrationData, AIFirstError> {
        // Generate performance predictions
        let performance_predictions = self
            .performance_predictor
            .predict_performance(&request)
            .await;

        // Configure service mesh routing
        let routing_info = self
            .service_mesh_integrator
            .configure_routing(&request)
            .await?;

        // Set up monitoring
        let monitoring_config = self.create_monitoring_configuration(&request);

        let registration_data = ServiceRegistrationData {
            service_id: request.service_id.clone(),
            registered_at: Utc::now(),
            routing_info,
            load_balancing_pool: Some("default".to_string()),
            monitoring_config,
            performance_predictions,
        };

        // Store registration information
        self.registered_services.insert(
            request.service_id.clone(),
            RegisteredServiceInfo {
                registration_request: request,
                registration_data: registration_data.clone(),
                registered_at: Utc::now(),
                last_health_check: None,
                current_status: ServiceStatus::Healthy,
            },
        );

        Ok(registration_data)
    }

    /// Create monitoring configuration for a service
    fn create_monitoring_configuration(
        &self,
        request: &UniversalServiceRegistrationRequest,
    ) -> MonitoringConfiguration {
        // Create adaptive monitoring configuration based on service type and requirements
        MonitoringConfiguration {
            metrics_interval_seconds: if request.resource_requirements.is_some() {
                30
            } else {
                60
            },
            health_check_frequency_seconds: if request.health_check.is_some() {
                15
            } else {
                30
            },
            performance_baseline: PerformanceBaseline {
                expected_response_time_ms: 100.0,
                expected_throughput_rps: 100.0,
                expected_error_rate_percentage: 1.0,
                expected_cpu_utilization_percentage: 50.0,
            },
            alert_thresholds: AlertThresholds {
                response_time_threshold_ms: 1000.0,
                error_rate_threshold_percentage: 5.0,
                cpu_utilization_threshold_percentage: 80.0,
                memory_utilization_threshold_percentage: 80.0,
            },
        }
    }
}
