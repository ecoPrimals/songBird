//! Basic Observability
//!
//! Implements:
//! - Metrics collection
//! - Event streaming
//! - Query API
//!
//! Modern Rust, integrates with existing event systems.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

mod events;
mod metrics;
mod query;

#[cfg(test)]
mod integration_tests;

pub use events::*;
pub use metrics::*;
pub use query::*;

/// Metric type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

/// Metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    pub name: Arc<str>,
    pub metric_type: MetricType,
    pub value: f64,
    pub timestamp: DateTime<Utc>,
    pub labels: HashMap<Arc<str>, Arc<str>>,
}

/// Metrics collector
pub struct MetricsCollector {
    metrics: Arc<RwLock<HashMap<Arc<str>, Vec<MetricValue>>>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a metric
    pub async fn record(&self, metric: MetricValue) {
        let mut metrics = self.metrics.write().await;
        metrics.entry(metric.name.clone()).or_insert_with(Vec::new).push(metric);
    }

    /// Get all metrics for a name
    pub async fn get_metrics(&self, name: &str) -> Vec<MetricValue> {
        let metrics = self.metrics.read().await;
        metrics.get(name).cloned().unwrap_or_default()
    }

    /// Get latest value for a metric
    pub async fn get_latest(&self, name: &str) -> Option<MetricValue> {
        let metrics = self.metrics.read().await;
        metrics.get(name).and_then(|v| v.last()).cloned()
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collection() {
        let collector = MetricsCollector::new();

        let metric = MetricValue {
            name: "test.counter".into(),
            metric_type: MetricType::Counter,
            value: 1.0,
            timestamp: Utc::now(),
            labels: HashMap::new(),
        };

        collector.record(metric).await;

        let metrics = collector.get_metrics("test.counter").await;
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].value, 1.0);
    }
}
