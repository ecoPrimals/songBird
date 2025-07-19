//! AI-Enhanced Service Mesh Types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// AI-Enhanced Service Health Check Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIServiceHealthData {
    pub service_id: String,
    pub health_status: ServiceHealthStatus,
    pub health_score: f64,
    pub health_prediction: HealthPrediction,
    pub resource_analysis: ResourceAnalysis,
    pub performance_metrics: ServicePerformanceMetrics,
    pub ai_recommendations: Vec<HealthRecommendation>,
    pub health_trends: HealthTrends,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceHealthStatus {
    Optimal,
    Healthy,
    Degraded { 
        issues: Vec<String>,
        severity: HealthSeverity,
    },
    Critical { 
        failures: Vec<String>,
        time_to_failure_estimate: Option<Duration>,
    },
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthPrediction {
    pub short_term_prediction: f64,
    pub medium_term_prediction: f64,
    pub long_term_prediction: f64,
    pub prediction_confidence: f64,
    pub prediction_factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAnalysis {
    pub cpu_analysis: ResourceMetric,
    pub memory_analysis: ResourceMetric,
    pub network_analysis: ResourceMetric,
    pub storage_analysis: ResourceMetric,
    pub efficiency_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetric {
    pub current_utilization: f64,
    pub average_utilization: f64,
    pub peak_utilization: f64,
    pub trend: UtilizationTrend,
    pub optimization_potential: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UtilizationTrend {
    Increasing,
    Stable,
    Decreasing,
    Volatile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePerformanceMetrics {
    pub response_time_metrics: ResponseTimeMetrics,
    pub throughput_metrics: ThroughputMetrics,
    pub error_metrics: ErrorMetrics,
    pub availability_metrics: AvailabilityMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeMetrics {
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub average: Duration,
    pub trend: MetricTrend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputMetrics {
    pub requests_per_second: f64,
    pub peak_rps: f64,
    pub average_rps: f64,
    pub trend: MetricTrend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    pub error_rate: f64,
    pub error_count: u64,
    pub error_types: HashMap<String, u64>,
    pub trend: MetricTrend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityMetrics {
    pub uptime_percentage: f64,
    pub downtime_duration: Duration,
    pub incident_count: u32,
    pub mttr: Duration, // Mean Time To Recovery
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricTrend {
    Improving,
    Stable,
    Degrading,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthRecommendation {
    pub recommendation_type: RecommendationType,
    pub priority: RecommendationPriority,
    pub description: String,
    pub estimated_impact: f64,
    pub implementation_effort: ImplementationEffort,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    ScaleUp,
    ScaleDown,
    OptimizeConfiguration,
    UpdateDependencies,
    RestartService,
    InvestigateAlerts,
    PerformMaintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Immediate,  // < 1 minute
    Quick,      // < 15 minutes
    Moderate,   // < 1 hour
    Significant, // < 1 day
    Major,      // > 1 day
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthTrends {
    pub hourly_trends: Vec<HealthDataPoint>,
    pub daily_trends: Vec<HealthDataPoint>,
    pub weekly_trends: Vec<HealthDataPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthDataPoint {
    pub timestamp: DateTime<Utc>,
    pub health_score: f64,
    pub incident_occurred: bool,
}

/// AI routing decision data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRoutingDecision {
    pub request_id: String,
    pub selected_service: ServiceEndpoint,
    pub routing_confidence: f64,
    pub decision_factors: Vec<RoutingFactor>,
    pub alternatives_considered: Vec<ServiceEndpoint>,
    pub prediction_accuracy: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub service_id: String,
    pub endpoint_url: String,
    pub health_score: f64,
    pub current_load: f64,
    pub estimated_response_time: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingFactor {
    pub factor_type: RoutingFactorType,
    pub weight: f64,
    pub impact_score: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingFactorType {
    Health,
    Performance,
    Load,
    Geography,
    Cost,
    Compliance,
    UserPreference,
}

/// Service mesh circuit breaker with AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AICircuitBreakerState {
    pub service_id: String,
    pub state: CircuitState,
    pub failure_rate: f64,
    pub ai_prediction: CircuitPrediction,
    pub recovery_recommendation: RecoveryStrategy,
    pub historical_patterns: Vec<CircuitEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CircuitState {
    Closed,
    Open { opened_at: DateTime<Utc> },
    HalfOpen { test_requests: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitPrediction {
    pub recovery_probability: f64,
    pub estimated_recovery_time: Option<Duration>,
    pub failure_root_cause: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStrategy {
    pub strategy_type: RecoveryType,
    pub steps: Vec<String>,
    pub estimated_duration: Duration,
    pub success_probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryType {
    AutomaticRetry,
    GradualRampUp,
    FallbackService,
    ManualIntervention,
    ServiceRestart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: CircuitEventType,
    pub context: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CircuitEventType {
    Opened,
    Closed,
    HalfOpenTriggered,
    TestRequestSucceeded,
    TestRequestFailed,
    RecoveryInitiated,
    RecoveryCompleted,
}

impl Default for HealthPrediction {
    fn default() -> Self {
        Self {
            short_term_prediction: 0.8,
            medium_term_prediction: 0.75,
            long_term_prediction: 0.7,
            prediction_confidence: 0.6,
            prediction_factors: Vec::new(),
        }
    }
}

impl Default for ServicePerformanceMetrics {
    fn default() -> Self {
        Self {
            response_time_metrics: ResponseTimeMetrics::default(),
            throughput_metrics: ThroughputMetrics::default(),
            error_metrics: ErrorMetrics::default(),
            availability_metrics: AvailabilityMetrics::default(),
        }
    }
}

impl Default for ResponseTimeMetrics {
    fn default() -> Self {
        Self {
            p50: Duration::from_millis(100),
            p95: Duration::from_millis(500),
            p99: Duration::from_millis(1000),
            average: Duration::from_millis(200),
            trend: MetricTrend::Stable,
        }
    }
}

impl Default for ThroughputMetrics {
    fn default() -> Self {
        Self {
            requests_per_second: 100.0,
            peak_rps: 150.0,
            average_rps: 80.0,
            trend: MetricTrend::Stable,
        }
    }
}

impl Default for ErrorMetrics {
    fn default() -> Self {
        Self {
            error_rate: 0.01, // 1%
            error_count: 10,
            error_types: HashMap::new(),
            trend: MetricTrend::Stable,
        }
    }
}

impl Default for AvailabilityMetrics {
    fn default() -> Self {
        Self {
            uptime_percentage: 99.9,
            downtime_duration: Duration::from_secs(60),
            incident_count: 2,
            mttr: Duration::from_minutes(15),
        }
    }
}

impl ServiceHealthStatus {
    pub fn is_healthy(&self) -> bool {
        matches!(self, Self::Optimal | Self::Healthy)
    }

    pub fn needs_attention(&self) -> bool {
        matches!(self, Self::Degraded { .. } | Self::Critical { .. })
    }
} 