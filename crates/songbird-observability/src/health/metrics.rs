/// Performance metrics and aggregation for health monitoring
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;

/// Health metrics aggregator
#[derive(Debug, Clone)]
pub struct HealthMetricsAggregator {
    /// Service performance metrics
    service_metrics: HashMap<Uuid, crate::health::types::ServicePerformanceMetrics>,

    /// Ecosystem-wide metrics
    ecosystem_metrics: EcosystemHealthMetrics,

    /// Capability-based metrics
    capability_metrics: HashMap<String, CapabilityHealthMetrics>,

    /// Category-based metrics
    category_metrics: HashMap<String, CategoryHealthMetrics>,

    /// Historical snapshots
    historical_snapshots: Vec<crate::health::types::HistoricalHealthSnapshot>,

    /// Maximum number of snapshots to keep
    max_snapshots: usize,
}

/// Ecosystem-wide health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemHealthMetrics {
    /// Total number of monitored services
    pub total_services: usize,

    /// Number of healthy services
    pub healthy_services: usize,

    /// Number of degraded services
    pub degraded_services: usize,

    /// Number of unhealthy services
    pub unhealthy_services: usize,

    /// Number of services with unknown status
    pub unknown_services: usize,

    /// Overall ecosystem health score (0.0 to 1.0)
    pub overall_health_score: f64,

    /// Average response time across all services
    pub avg_response_time_ms: f64,

    /// Average success rate across all services
    pub avg_success_rate: f64,

    /// Last updated timestamp
    pub last_updated: SystemTime,
}

impl Default for EcosystemHealthMetrics {
    fn default() -> Self {
        Self {
            total_services: 0,
            healthy_services: 0,
            degraded_services: 0,
            unhealthy_services: 0,
            unknown_services: 0,
            overall_health_score: 1.0,
            avg_response_time_ms: 0.0,
            avg_success_rate: 1.0,
            last_updated: SystemTime::now(),
        }
    }
}

/// Capability-based health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityHealthMetrics {
    /// Capability name
    pub capability_name: String,

    /// Services providing this capability
    pub service_count: usize,

    /// Healthy services for this capability
    pub healthy_count: usize,

    /// Average response time for this capability
    pub avg_response_time_ms: f64,

    /// Success rate for this capability
    pub success_rate: f64,

    /// Capability health score (0.0 to 1.0)
    pub health_score: f64,

    /// Last updated timestamp
    pub last_updated: SystemTime,
}

/// Category-based health metrics (storage, compute, ai, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryHealthMetrics {
    /// Category name
    pub category_name: String,

    /// Services in this category
    pub service_count: usize,

    /// Healthy services in this category
    pub healthy_count: usize,

    /// Average response time for this category
    pub avg_response_time_ms: f64,

    /// Success rate for this category
    pub success_rate: f64,

    /// Category health score (0.0 to 1.0)
    pub health_score: f64,

    /// Last updated timestamp
    pub last_updated: SystemTime,
}

impl HealthMetricsAggregator {
    /// Create a new metrics aggregator
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            service_metrics: HashMap::new(),
            ecosystem_metrics: EcosystemHealthMetrics::default(),
            capability_metrics: HashMap::new(),
            category_metrics: HashMap::new(),
            historical_snapshots: Vec::new(),
            max_snapshots,
        }
    }

    /// Update metrics for a service
    pub fn update_service_metrics(
        &mut self,
        service_id: Uuid,
        metrics: crate::health::types::ServicePerformanceMetrics,
    ) {
        self.service_metrics.insert(service_id, metrics);
        self.recalculate_ecosystem_metrics();
    }

    /// Update capability metrics
    pub fn update_capability_metrics(
        &mut self,
        capability: String,
        metrics: CapabilityHealthMetrics,
    ) {
        self.capability_metrics.insert(capability, metrics);
    }

    /// Update category metrics
    pub fn update_category_metrics(&mut self, category: String, metrics: CategoryHealthMetrics) {
        self.category_metrics.insert(category, metrics);
    }

    /// Get ecosystem metrics
    pub fn get_ecosystem_metrics(&self) -> &EcosystemHealthMetrics {
        &self.ecosystem_metrics
    }

    /// Get metrics for a specific service
    pub fn get_service_metrics(
        &self,
        service_id: &Uuid,
    ) -> Option<&crate::health::types::ServicePerformanceMetrics> {
        self.service_metrics.get(service_id)
    }

    /// Get metrics for a capability
    pub fn get_capability_metrics(&self, capability: &str) -> Option<&CapabilityHealthMetrics> {
        self.capability_metrics.get(capability)
    }

    /// Get metrics for a category
    pub fn get_category_metrics(&self, category: &str) -> Option<&CategoryHealthMetrics> {
        self.category_metrics.get(category)
    }

    /// Take a historical snapshot
    pub fn take_snapshot(
        &mut self,
        healthy_services: Vec<Uuid>,
        degraded_services: Vec<Uuid>,
        unhealthy_services: Vec<Uuid>,
        unknown_services: Vec<Uuid>,
    ) {
        let snapshot = crate::health::types::HistoricalHealthSnapshot {
            timestamp: SystemTime::now(),
            healthy_services,
            degraded_services,
            unhealthy_services,
            unknown_services,
            health_score: self.ecosystem_metrics.overall_health_score,
            performance_snapshot: self.service_metrics.clone(),
        };

        self.historical_snapshots.push(snapshot);

        // Keep only the most recent snapshots
        if self.historical_snapshots.len() > self.max_snapshots {
            self.historical_snapshots.remove(0);
        }
    }

    /// Get historical snapshots
    pub fn get_historical_snapshots(&self) -> &[crate::health::types::HistoricalHealthSnapshot] {
        &self.historical_snapshots
    }

    /// Recalculate ecosystem-wide metrics
    fn recalculate_ecosystem_metrics(&mut self) {
        let total_services = self.service_metrics.len();

        if total_services == 0 {
            self.ecosystem_metrics = EcosystemHealthMetrics::default();
            return;
        }

        let mut total_response_time = 0.0;
        let mut total_success_rate = 0.0;

        for metrics in self.service_metrics.values() {
            total_response_time += metrics.avg_response_time_ms;
            total_success_rate += metrics.success_rate;
        }

        self.ecosystem_metrics.total_services = total_services;
        self.ecosystem_metrics.avg_response_time_ms = total_response_time / total_services as f64;
        self.ecosystem_metrics.avg_success_rate = total_success_rate / total_services as f64;
        self.ecosystem_metrics.last_updated = SystemTime::now();

        // Calculate overall health score based on success rate and response time
        let response_time_score = if self.ecosystem_metrics.avg_response_time_ms < 100.0 {
            1.0
        } else if self.ecosystem_metrics.avg_response_time_ms < 1000.0 {
            0.8
        } else if self.ecosystem_metrics.avg_response_time_ms < 5000.0 {
            0.5
        } else {
            0.2
        };

        self.ecosystem_metrics.overall_health_score =
            (self.ecosystem_metrics.avg_success_rate * 0.7) + (response_time_score * 0.3);
    }

    /// Calculate performance trend for a service
    pub fn calculate_performance_trend(
        &self,
        service_id: &Uuid,
        window_minutes: u64,
    ) -> Option<crate::health::types::PerformanceTrend> {
        // This is a simplified implementation
        // In a real system, this would analyze historical data points
        self.service_metrics.get(service_id).map(|_metrics| {
            crate::health::types::PerformanceTrend {
                direction: crate::health::types::TrendDirection::Stable,
                strength: 0.5,
                window_duration: std::time::Duration::from_secs(window_minutes * 60),
                confidence: 0.8,
            }
        })
    }
}
