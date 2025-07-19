//! Performance predictions and risk assessment

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Workload performance predictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadPerformancePredictions {
    /// Response time predictions
    pub response_time: ResponseTimeDistribution,

    /// Throughput predictions
    pub throughput: ThroughputPrediction,

    /// Resource utilization predictions
    pub resource_utilization: ResourceUtilizationPrediction,

    /// Cost predictions
    pub cost: CostPrediction,

    /// SLA compliance predictions
    pub sla_compliance: SLACompliancePrediction,
}

/// Response time distribution predictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeDistribution {
    /// 50th percentile (median) response time in ms
    pub p50_ms: f64,

    /// 90th percentile response time in ms
    pub p90_ms: f64,

    /// 95th percentile response time in ms
    pub p95_ms: f64,

    /// 99th percentile response time in ms
    pub p99_ms: f64,

    /// Maximum predicted response time in ms
    pub max_ms: f64,
}

/// Throughput prediction details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputPrediction {
    /// Expected throughput (requests per second)
    pub expected_rps: f64,

    /// Peak throughput capability
    pub peak_rps: f64,

    /// Sustained throughput capability
    pub sustained_rps: f64,
}

/// Resource utilization predictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilizationPrediction {
    /// CPU utilization prediction
    pub cpu: UtilizationRange,

    /// Memory utilization prediction
    pub memory: UtilizationRange,

    /// Storage utilization prediction
    pub storage: UtilizationRange,

    /// Network utilization prediction
    pub network: UtilizationRange,
}

/// Utilization range prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilizationRange {
    /// Minimum utilization percentage
    pub min_percentage: f64,

    /// Average utilization percentage
    pub avg_percentage: f64,

    /// Peak utilization percentage
    pub peak_percentage: f64,
}

/// Cost prediction analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostPrediction {
    /// Estimated cost per hour
    pub cost_per_hour: f64,

    /// Estimated monthly cost
    pub monthly_cost: f64,

    /// Cost breakdown by resource type
    pub cost_breakdown: Vec<String>,

    /// Cost optimization opportunities
    pub optimization_opportunities: Vec<String>,
}

/// SLA compliance prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLACompliancePrediction {
    /// Predicted availability percentage
    pub availability_percentage: f64,

    /// Predicted performance compliance
    pub performance_compliance_percentage: f64,

    /// Risk factors for SLA breaches
    pub risk_factors: Vec<String>,

    /// Mitigation strategies
    pub mitigation_strategies: Vec<String>,
}

/// Workload risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadRiskAssessment {
    /// Overall risk score (0.0 - 1.0)
    pub overall_risk_score: f64,

    /// Individual risk factors
    pub risk_factors: Vec<RiskFactor>,

    /// Risk mitigation plan
    pub mitigation_plan: RiskMitigationPlan,

    /// Risk monitoring requirements
    pub monitoring_requirements: Vec<String>,
}

/// Individual risk factor analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    /// Risk factor name
    pub name: String,

    /// Risk score for this factor (0.0 - 1.0)
    pub score: f64,

    /// Description of the risk
    pub description: String,

    /// Impact if risk materializes
    pub impact: String,

    /// Likelihood of occurrence
    pub likelihood: f64,

    /// Recommended actions
    pub recommended_actions: Vec<String>,
}

/// Risk mitigation plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMitigationPlan {
    /// Preventive measures
    pub preventive_measures: Vec<String>,

    /// Contingency plans
    pub contingency_plans: Vec<String>,

    /// Recovery procedures
    pub recovery_procedures: Vec<String>,

    /// Monitoring and alerting setup
    pub monitoring_setup: Vec<String>,
}

/// Processing timeline information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingTimeline {
    /// Estimated start time
    pub estimated_start: DateTime<Utc>,

    /// Estimated completion time  
    pub estimated_completion: DateTime<Utc>,

    /// Processing phases
    pub phases: Vec<ProcessingPhase>,

    /// Critical path dependencies
    pub critical_dependencies: Vec<String>,
}

/// Individual processing phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingPhase {
    /// Phase name
    pub name: String,

    /// Phase description
    pub description: String,

    /// Estimated duration
    pub estimated_duration_ms: u64,

    /// Phase dependencies
    pub dependencies: Vec<String>,

    /// Resource requirements
    pub resource_requirements: std::collections::HashMap<String, f64>,

    /// Success criteria
    pub success_criteria: Vec<String>,
}
