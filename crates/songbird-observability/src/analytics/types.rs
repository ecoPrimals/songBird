// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Analytics type definitions for production monitoring
//!
//! Data points, time series, trend analysis, anomaly detection,
//! prediction results, and analytics configuration.

#![allow(missing_docs, reason = "analytics DTOs mirror wire and JSON field names")]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

pub(crate) mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(d: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(d.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

/// Analytics data point
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataPoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Metric name
    pub metric_name: String,
    /// Metric value
    pub value: f64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Time series data
#[derive(Debug, Clone)]
pub struct TimeSeries {
    /// Series name
    pub name: String,
    /// Data points
    pub data_points: VecDeque<DataPoint>,
    /// Maximum data points to retain
    pub max_points: usize,
}

/// Trend analysis result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrendAnalysis {
    /// Trend direction
    pub direction: TrendDirection,
    /// Trend strength (0.0 to 1.0)
    pub strength: f64,
    /// Trend duration
    #[serde(with = "duration_serde")]
    pub duration: Duration,
    /// Confidence in analysis
    pub confidence: f64,
    /// Analysis timestamp
    pub analyzed_at: DateTime<Utc>,
}

/// Trend directions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnomalyResult {
    /// Anomaly type
    pub anomaly_type: AnomalyType,
    /// Severity level
    pub severity: AnomalySeverity,
    /// Anomaly score
    pub score: f64,
    /// Expected value
    pub expected_value: f64,
    /// Actual value
    pub actual_value: f64,
    /// Detection timestamp
    pub detected_at: DateTime<Utc>,
    /// Description
    pub description: String,
}

/// Anomaly types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalyType {
    Spike,
    Drop,
    Drift,
    Oscillation,
    Flatline,
}

/// Anomaly severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Prediction result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PredictionResult {
    /// Predicted metric
    pub metric_name: String,
    /// Predicted value
    pub predicted_value: f64,
    /// Prediction confidence
    pub confidence: f64,
    /// Prediction horizon
    #[serde(with = "duration_serde")]
    pub horizon: Duration,
    /// Prediction timestamp
    pub predicted_at: DateTime<Utc>,
    /// Prediction method used
    pub method: PredictionMethod,
}

/// Prediction methods
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredictionMethod {
    LinearRegression,
    ExponentialSmoothing,
    MovingAverage,
    SeasonalDecomposition,
    NeuralNetwork,
}

/// Anomaly detection model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnomalyModel {
    /// Model name
    pub name: String,
    /// Statistical baseline
    pub baseline_mean: f64,
    /// Statistical standard deviation
    pub baseline_std: f64,
    /// Threshold multiplier for anomaly detection
    pub threshold_multiplier: f64,
    /// Model training data size
    pub training_size: usize,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

/// Prediction model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PredictionModel {
    /// Model name
    pub name: String,
    /// Model coefficients (for linear regression)
    pub coefficients: Vec<f64>,
    /// Model accuracy
    pub accuracy: f64,
    /// Training data size
    pub training_size: usize,
    /// Last training timestamp
    pub last_trained: DateTime<Utc>,
}

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyticsConfig {
    /// Maximum data points per time series
    pub max_data_points: usize,
    /// Anomaly detection sensitivity
    pub anomaly_sensitivity: f64,
    /// Prediction horizon
    #[serde(with = "duration_serde")]
    pub prediction_horizon: Duration,
    /// Model retraining interval
    #[serde(with = "duration_serde")]
    pub retraining_interval: Duration,
    /// Enable real-time processing
    pub enable_real_time: bool,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            max_data_points: 10000,
            anomaly_sensitivity: 2.0,
            prediction_horizon: Duration::from_secs(3600),
            retraining_interval: Duration::from_secs(86400),
            enable_real_time: true,
        }
    }
}

/// Analytics statistics
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnalyticsStatistics {
    /// Total data points ingested
    pub total_data_points: u64,
    /// Trends analyzed
    pub trends_analyzed: u64,
    /// Anomalies detected (non-low severity)
    pub anomalies_detected: u64,
    /// Predictions made
    pub predictions_made: u64,
    /// Models trained
    pub models_trained: u64,
    /// Average processing time
    pub avg_processing_time: Duration,
}

/// Analytics insight
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnalyticsInsight {
    /// Insight type
    pub insight_type: InsightType,
    /// Insight title
    pub title: String,
    /// Insight description
    pub description: String,
    /// Severity level
    pub severity: AnomalySeverity,
    /// Recommended action
    pub recommendation: String,
    /// Generation timestamp
    pub generated_at: DateTime<Utc>,
}

/// Insight types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InsightType {
    Trend,
    Anomaly,
    Prediction,
    Performance,
    Resource,
}
