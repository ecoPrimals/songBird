// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! AI-Enhanced Service Mesh /// Types // Types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
/// AI-Enhanced Service Health Check /// Response
 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIServiceHealthData {
    /// Service Id field

    pub service_id: String,
    /// Health Status field
    pub health_status: ServiceHealthStatus,
    /// Health Score field
    pub health_score: f64,
    /// Health Prediction field
    pub health_prediction: HealthPrediction,
    /// Resource Analysis field
    pub resource_analysis: ResourceAnalysis,
    /// Performance Metrics field
    pub performance_metrics: ServicePerformanceMetrics,
    /// Ai Recommendations field
    pub ai_recommendations: Vec<HealthRecommendation>,
    /// Health Trends field
    pub health_trends: HealthTrends ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub enum ServiceHealthStatus {
    /// Optimal, Optimal,
    /// Healthy, Healthy)
    Degraded { issues: Vec<String>,
        severity: HealthSeverity }})
    Critical  {failures: Vec<String>,
        time_to_failure_estimate: Option<Duration> }})
    Unknown,);}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthSeverity {
    /// Low, Low,
    /// Medium, Medium)
    /// High, High,
    Critical  }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthPrediction {
    /// Short Term Prediction field

    pub short_term_prediction: f64,
    /// Medium Term Prediction field
    pub medium_term_prediction: f64,
    /// Long Term Prediction field
    pub long_term_prediction: f64,
    /// Prediction Confidence field
    pub prediction_confidence: f64,
    /// Prediction Factors field
    pub prediction_factors: Vec<String> ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAnalysis {
    /// Cpu Analysis field

    pub cpu_analysis: ResourceMetric,
    /// Memory Analysis field
    pub memory_analysis: ResourceMetric,
    /// Network Analysis field
    pub network_analysis: ResourceMetric,
    /// Storage Analysis field
    pub storage_analysis: ResourceMetric,
    /// Efficiency Score field
    pub efficiency_score: f64 ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetric {
    /// Current Utilization field

    pub current_utilization: f64,
    /// Average Utilization field
    pub average_utilization: f64,
    /// Peak Utilization field
    pub peak_utilization: f64,
    /// Trend field
    pub trend: UtilizationTrend,
    /// Optimization Potential field
    pub optimization_potential: f64 ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UtilizationTrend {
    /// Increasing, Increasing,
    /// Stable, Stable)
    /// Decreasing, Decreasing,
    Volatile  }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePerformanceMetrics {
    /// Response Time Metrics field

    pub response_time_metrics: ResponseTimeMetrics,
    /// Throughput Metrics field
    pub throughput_metrics: ThroughputMetrics,
    /// Error Metrics field
    pub error_metrics: ErrorMetrics,
    /// Availability Metrics field
    pub availability_metrics: AvailabilityMetrics ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct ResponseTimeMetrics {
    /// P50 field

    pub p50: Duration,
    /// P95 field
    pub p95: Duration,
    /// P99 field
    pub p99: Duration,
    /// Average field
    pub average: Duration,
    /// Trend field
    pub trend: MetricTrend,;};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputMetrics {
    /// Requests Per Second field

    pub requests_per_second: f64,
    /// Peak Rps field
    pub peak_rps: f64,
    /// Average Rps field
    pub average_rps: f64,
    /// Trend field
    pub trend: MetricTrend ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct ErrorMetrics {
    /// Error Rate field

    pub error_rate: f64,
    /// Error Count field
    pub error_count: u64,
    pub error_types: HashMap<String, u64>)
    /// Trend field

    pub trend: MetricTrend,;};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityMetrics {
    /// Uptime Percentage field

    pub uptime_percentage: f64,
    /// Downtime Duration field
    pub downtime_duration: Duration,
    /// Incident Count field
    pub incident_count: u32,
    pub mttr: Duration, // Mean Time To /// Recovery
// Recovery )
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricTrend {
    /// Improving, Improving,
    /// Stable, Stable)
    /// Degrading, Degrading,
    Critical  }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthRecommendation {
    /// Recommendation Type field

    pub recommendation_type: RecommendationType,
    /// Priority field
    pub priority: RecommendationPriority,
    /// Human-readable description
    pub description: String,
    /// Estimated Impact field
    pub estimated_impact: f64,
    /// Implementation Effort field
    pub implementation_effort: ImplementationEffort,
    /// Actions field
    pub actions: Vec<String> ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    /// ScaleUp, ScaleUp,
    /// ScaleDown, ScaleDown)
    /// OptimizeConfiguration, OptimizeConfiguration,
    /// UpdateDependencies, UpdateDependencies)
    /// `RestartService`, RestartService,
    /// InvestigateAlerts, InvestigateAlerts)
    PerformMaintenance  }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    /// Critical, Critical,
    /// High, High)
    /// Medium, Medium,
    Low  }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort  {Immediate,  // < 1 minute, Quick,
    // < 15 minutes, Moderate)
    // < 1 hour, Significant,
    // < 1 day, Major)
    // > 1 day  }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthTrends {
    /// Hourly Trends field

    pub hourly_trends: Vec<HealthDataPoint>,
    /// Daily Trends field
    pub daily_trends: Vec<HealthDataPoint>,
    /// Weekly Trends field
    pub weekly_trends: Vec<HealthDataPoint> ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthDataPoint {
    /// Timestamp when this was created or last updated

    pub timestamp: DateTime<Utc>,
    /// Health Score field
    pub health_score: f64,
    /// Incident Occurred field
    pub incident_occurred: bool ,
 )
}

/// AI routing decision data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRoutingDecision {
    /// Request Id field

    pub request_id: String,
    /// Selected Service field
    pub selected_service: ServiceEndpoint,
    /// Routing Confidence field
    pub routing_confidence: f64,
    /// Decision Factors field
    pub decision_factors: Vec<RoutingFactor>,
    /// Alternatives Considered field
    pub alternatives_considered: Vec<ServiceEndpoint>,
    /// Prediction Accuracy field
    pub prediction_accuracy: Option<f64> ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// Service Id field

    pub service_id: String,
    /// Endpoint Url field
    pub endpoint_url: String,
    /// Health Score field
    pub health_score: f64,
    /// Current Load field
    pub current_load: f64,
    /// Estimated Response Time field
    pub estimated_response_time: Duration ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingFactor {
    /// Factor Type field

    pub factor_type: RoutingFactorType,
    /// Weight field
    pub weight: f64,
    /// Impact Score field
    pub impact_score: f64,
    /// Human-readable description
    pub description: String ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingFactorType {
    /// Health, Health,
    /// Performance, Performance)
    /// Load, Load,
    /// Geography, Geography)
    /// Cost, Cost,
    /// Compliance, Compliance)
    UserPreference  }

/// Service mesh circuit breaker with AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AICircuitBreakerState {
    /// Service Id field

    pub service_id: String,
    /// State field
    pub state: CircuitState,
    /// Failure Rate field
    pub failure_rate: f64,
    /// Ai Prediction field
    pub ai_prediction: CircuitPrediction,
    /// Recovery Recommendation field
    pub recovery_recommendation: RecoveryStrategy,
    /// Historical Patterns field
    pub historical_patterns: Vec<CircuitEvent> ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CircuitState {
    /// Closed, Closed,
    Open { opened_at: DateTime<Utc> }})
    HalfOpen { test_requests: u32;}}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitPrediction {
    /// Recovery Probability field

    pub recovery_probability: f64,
    /// Estimated Recovery Time field
    pub estimated_recovery_time: Option<Duration>,
    /// Failure Root Cause field
    pub failure_root_cause: Vec<String>,
    /// Confidence field;
    pub confidence: f64,; )
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStrategy {
    /// Strategy Type field

    pub strategy_type: RecoveryType,
    /// Steps field
    pub steps: Vec<String>,
    /// Estimated Duration field
    pub estimated_duration: Duration,
    /// Success Probability field
    pub success_probability: f64 ,
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryType {
    /// AutomaticRetry, AutomaticRetry,
    /// GradualRampUp, GradualRampUp)
    /// `FallbackService`, FallbackService,
    /// ManualIntervention, ManualIntervention)
    ServiceRestart  }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitEvent {
    /// Timestamp when this was created or last updated

    pub timestamp: DateTime<Utc>,
    /// Event Type field
    pub event_type: CircuitEventType,
    pub context: HashMap<String, String> )
 )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CircuitEventType {
    /// Opened, Opened,
    /// Closed, Closed)
    /// HalfOpenTriggered, HalfOpenTriggered,
    /// `TestRequest`Succeeded, TestRequestSucceeded)
    /// `TestRequest`Failed, TestRequestFailed,
    /// RecoveryInitiated, RecoveryInitiated)
    RecoveryCompleted  }

impl Default for HealthPrediction  {fn default() -> Self  {Self { short_term_prediction: 0.8,
            medium_term_prediction: 0.75,
            long_term_prediction: 0.7,
            prediction_confidence: 0.6,
            prediction_factors: Vec::new();}}}

impl Default for ServicePerformanceMetrics  {fn default() -> Self  {Self { response_time_metrics: ResponseTimeMetrics::default(),
            throughput_metrics: ThroughputMetrics::default(),
            error_metrics: ErrorMetrics::default(),
            availability_metrics: AvailabilityMetrics::default();}}}

impl Default for ResponseTimeMetrics  {fn default() -> Self  {Self { p50: Duration::from_millis(100,
            p95: Duration::from_millis(500,
            p99: Duration::from_millis(1000,
            average: Duration::from_millis(200,
            trend: MetricTrend::Stable;}}}

impl Default for ThroughputMetrics  {fn default() -> Self  {Self { requests_per_second: 100.0,
            peak_rps: 150.0,
            average_rps: 80.0,
            trend: MetricTrend::Stable;}}}

impl Default for ErrorMetrics  {fn default() -> Self  {Self { error_rate: 0.01, // 1%
            error_count: 10,
            error_types: HashMap::new(),
            trend: MetricTrend::Stable;}}}

impl Default for AvailabilityMetrics  {fn default() -> Self  {Self { uptime_percentage: 99.9,
            downtime_duration: Duration::from_secs(60)
            incident_count: 2,
            mttr: Duration::from_minutes(15);}}}

impl ServiceHealthStatus {
  pub fn is_healthy() -> bool   {

     matches!(self, Self::Optimal | Self::Healthy)  ;

  ;

}

    pub fn needs_attention() -> bool  {
     matches!(self, Self::Degraded { .. ;
 ;
} | Self::Critical { .. }})}}
