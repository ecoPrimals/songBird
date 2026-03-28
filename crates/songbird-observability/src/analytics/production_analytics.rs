// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Production Analytics Engine
//!
//! Real data processing and machine learning analytics for trend analysis,
//! anomaly detection, and metric prediction. Type definitions live in
//! [`super::types`].

#![allow(missing_docs)]

pub use super::types::*;

use chrono::Utc;
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Production analytics engine
pub struct ProductionAnalyticsEngine {
    time_series: Arc<RwLock<HashMap<String, TimeSeries>>>,
    trend_cache: Arc<RwLock<HashMap<String, TrendAnalysis>>>,
    anomaly_models: Arc<RwLock<HashMap<String, AnomalyModel>>>,
    prediction_models: Arc<RwLock<HashMap<String, PredictionModel>>>,
    config: AnalyticsConfig,
    stats: Arc<RwLock<AnalyticsStatistics>>,
}

impl ProductionAnalyticsEngine {
    /// Create new production analytics engine
    #[must_use]
    pub fn new(config: AnalyticsConfig) -> Self {
        Self {
            time_series: Arc::new(RwLock::new(HashMap::new())),
            trend_cache: Arc::new(RwLock::new(HashMap::new())),
            anomaly_models: Arc::new(RwLock::new(HashMap::new())),
            prediction_models: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(AnalyticsStatistics::default())),
        }
    }

    /// Add data point for analysis
    ///
    /// # Errors
    ///
    /// Returns an error if internal storage or real-time anomaly checks fail.
    pub async fn add_data_point(&self, data_point: DataPoint) -> SongbirdResult<()> {
        let metric_name = data_point.metric_name.clone();

        let mut series_map = self.time_series.write().await;
        let series = series_map.entry(metric_name.clone()).or_insert_with(|| TimeSeries {
            name: metric_name.clone(),
            data_points: VecDeque::new(),
            max_points: self.config.max_data_points,
        });

        series.data_points.push_back(data_point.clone());

        if series.data_points.len() > series.max_points {
            series.data_points.pop_front();
        }

        let mut stats = self.stats.write().await;
        stats.total_data_points += 1;

        if self.config.enable_real_time {
            drop(series_map);
            drop(stats);

            if let Ok(anomaly) = self.detect_anomaly(&metric_name, data_point.value).await
                && anomaly.severity != AnomalySeverity::Low
            {
                info!("Anomaly detected: {:?} - {}", anomaly.anomaly_type, anomaly.description);
            }
        }

        debug!("Added data point for metric: {} (value: {})", metric_name, data_point.value);
        Ok(())
    }

    /// Analyze trends for a metric
    ///
    /// # Errors
    ///
    /// Returns an error when the metric is unknown or there are too few data points for the window.
    pub async fn analyze_trends(
        &self,
        metric_name: &str,
        window_size: usize,
    ) -> SongbirdResult<TrendAnalysis> {
        let series_map = self.time_series.read().await;
        let series = series_map
            .get(metric_name)
            .ok_or_else(|| SongbirdError::service("analytics_engine", "Unknown metric"))?;

        if series.data_points.len() < window_size {
            return Err(SongbirdError::service(
                "analytics_engine",
                "Insufficient data for trend analysis",
            ));
        }

        let recent_points: Vec<&DataPoint> =
            series.data_points.iter().rev().take(window_size).collect();

        let (slope, r_squared) = Self::calculate_linear_regression(&recent_points);

        let direction = if slope > 0.01 {
            TrendDirection::Increasing
        } else if slope < -0.01 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        };

        let strength = r_squared.abs();
        let confidence = if r_squared > 0.7 {
            0.9
        } else if r_squared > 0.5 {
            0.7
        } else {
            0.5
        };

        let trend_analysis = TrendAnalysis {
            direction,
            strength,
            duration: Duration::from_secs((window_size * 60) as u64),
            confidence,
            analyzed_at: Utc::now(),
        };

        let mut cache = self.trend_cache.write().await;
        cache.insert(metric_name.to_string(), trend_analysis.clone());

        let mut stats = self.stats.write().await;
        stats.trends_analyzed += 1;

        info!(
            "Trend analysis complete for {}: {:?} (strength: {:.2})",
            metric_name, direction, strength
        );

        Ok(trend_analysis)
    }

    /// Detect anomalies in real-time
    ///
    /// # Errors
    ///
    /// Returns an error if concurrent access to internal state fails.
    pub async fn detect_anomaly(
        &self,
        metric_name: &str,
        value: f64,
    ) -> SongbirdResult<AnomalyResult> {
        let model = {
            let models = self.anomaly_models.read().await;
            if let Some(model) = models.get(metric_name) {
                model.clone()
            } else {
                drop(models);
                let new_model = self.create_anomaly_model(metric_name, value);
                let mut models = self.anomaly_models.write().await;
                models.insert(metric_name.to_string(), new_model.clone());
                new_model
            }
        };

        let z_score = if model.baseline_std.abs() < f64::EPSILON {
            0.0
        } else {
            (value - model.baseline_mean) / model.baseline_std
        };
        let anomaly_score = z_score.abs();

        let (anomaly_type, severity) = if anomaly_score > model.threshold_multiplier * 3.0 {
            (
                if z_score > 0.0 {
                    AnomalyType::Spike
                } else {
                    AnomalyType::Drop
                },
                AnomalySeverity::Critical,
            )
        } else if anomaly_score > model.threshold_multiplier * 2.0 {
            (
                if z_score > 0.0 {
                    AnomalyType::Spike
                } else {
                    AnomalyType::Drop
                },
                AnomalySeverity::High,
            )
        } else if anomaly_score > model.threshold_multiplier {
            (AnomalyType::Drift, AnomalySeverity::Medium)
        } else {
            (AnomalyType::Drift, AnomalySeverity::Low)
        };

        let anomaly = AnomalyResult {
            anomaly_type,
            severity,
            score: anomaly_score,
            expected_value: model.baseline_mean,
            actual_value: value,
            detected_at: Utc::now(),
            description: format!(
                "Value {value} deviates from expected {} by {anomaly_score:.2} standard deviations",
                model.baseline_mean
            ),
        };

        if severity != AnomalySeverity::Low {
            let mut stats = self.stats.write().await;
            stats.anomalies_detected += 1;
        }

        Ok(anomaly)
    }

    fn create_anomaly_model(&self, metric_name: &str, initial_value: f64) -> AnomalyModel {
        let std = (initial_value * 0.1).max(1e-9);
        AnomalyModel {
            name: metric_name.to_string(),
            baseline_mean: initial_value,
            baseline_std: std,
            threshold_multiplier: self.config.anomaly_sensitivity,
            training_size: 1,
            last_updated: Utc::now(),
        }
    }

    /// Make predictions for a metric
    ///
    /// # Errors
    ///
    /// Returns an error when the metric is unknown or there is insufficient history for a forecast.
    pub async fn predict_metric(
        &self,
        metric_name: &str,
        horizon: Duration,
    ) -> SongbirdResult<PredictionResult> {
        let series_map = self.time_series.read().await;
        let series = series_map
            .get(metric_name)
            .ok_or_else(|| SongbirdError::service("analytics_engine", "Unknown metric"))?;

        if series.data_points.len() < 10 {
            return Err(SongbirdError::service(
                "analytics_engine",
                "Insufficient data for prediction",
            ));
        }

        let recent_points: Vec<&DataPoint> = series.data_points.iter().rev().take(50).collect();

        let (slope, r_squared) = Self::calculate_linear_regression(&recent_points);

        let horizon_minutes = horizon.as_secs_f64() / 60.0;
        let last_value = recent_points.first().map_or(0.0, |p| p.value);
        let predicted_value = slope.mul_add(horizon_minutes, last_value);

        let prediction = PredictionResult {
            metric_name: metric_name.to_string(),
            predicted_value,
            confidence: r_squared,
            horizon,
            predicted_at: Utc::now(),
            method: PredictionMethod::LinearRegression,
        };

        let mut stats = self.stats.write().await;
        stats.predictions_made += 1;

        info!(
            "Prediction for {}: {:.2} (confidence: {:.2})",
            metric_name, predicted_value, r_squared
        );

        Ok(prediction)
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "Analytics: index and sample count cast to f64 for regression math"
    )]
    fn calculate_linear_regression(data_points: &[&DataPoint]) -> (f64, f64) {
        if data_points.len() < 2 {
            return (0.0, 0.0);
        }

        let n = data_points.len() as f64;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_x_times_y = 0.0;
        let mut sum_x2 = 0.0;

        for (i, point) in data_points.iter().enumerate() {
            let x = i as f64;
            let y = point.value;

            sum_x += x;
            sum_y += y;
            sum_x_times_y += x * y;
            sum_x2 += x * x;
        }

        let denominator = n.mul_add(sum_x2, -(sum_x * sum_x));
        if denominator.abs() < f64::EPSILON {
            return (0.0, 0.0);
        }

        let slope = n.mul_add(sum_x_times_y, -(sum_x * sum_y)) / denominator;

        let mean_y = sum_y / n;
        let mut ss_tot = 0.0;
        let mut ss_res = 0.0;

        for (i, point) in data_points.iter().enumerate() {
            let x = i as f64;
            let y = point.value;
            let intercept = (sum_y - slope * sum_x) / n;
            let y_pred = slope.mul_add(x, intercept);

            ss_tot += (y - mean_y).powi(2);
            ss_res += (y - y_pred).powi(2);
        }

        let r_squared = if ss_tot > 0.0 {
            1.0 - ss_res / ss_tot
        } else {
            0.0
        };

        (slope, r_squared)
    }

    /// Train anomaly detection model
    ///
    /// # Errors
    ///
    /// Returns an error when the metric is unknown or there is insufficient training data.
    #[expect(
        clippy::cast_precision_loss,
        reason = "Analytics: sample count cast to f64 for mean and variance"
    )]
    pub async fn train_anomaly_model(&self, metric_name: &str) -> SongbirdResult<()> {
        let series_map = self.time_series.read().await;
        let series = series_map
            .get(metric_name)
            .ok_or_else(|| SongbirdError::service("analytics_engine", "Unknown metric"))?;

        if series.data_points.len() < 30 {
            return Err(SongbirdError::service(
                "analytics_engine",
                "Insufficient data for model training",
            ));
        }

        let values: Vec<f64> = series.data_points.iter().map(|p| p.value).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;

        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt().max(1e-9);

        let model = AnomalyModel {
            name: metric_name.to_string(),
            baseline_mean: mean,
            baseline_std: std_dev,
            threshold_multiplier: self.config.anomaly_sensitivity,
            training_size: values.len(),
            last_updated: Utc::now(),
        };

        let mut models = self.anomaly_models.write().await;
        models.insert(metric_name.to_string(), model);

        let mut stats = self.stats.write().await;
        stats.models_trained += 1;

        info!("Trained anomaly model for {}: mean={:.2}, std={:.2}", metric_name, mean, std_dev);

        Ok(())
    }

    /// Generate analytics insights
    ///
    /// # Errors
    ///
    /// Returns an error if underlying trend or prediction calls fail.
    pub async fn generate_insights(
        &self,
        metric_name: &str,
    ) -> SongbirdResult<Vec<AnalyticsInsight>> {
        let mut insights = Vec::new();

        if let Ok(trend) = self.analyze_trends(metric_name, 20).await {
            insights.push(AnalyticsInsight {
                insight_type: InsightType::Trend,
                title: format!("Trend Analysis for {metric_name}"),
                description: format!(
                    "Metric shows {:?} trend with {:.1}% strength",
                    trend.direction,
                    trend.strength * 100.0
                ),
                severity: if trend.strength > 0.8 {
                    AnomalySeverity::High
                } else {
                    AnomalySeverity::Medium
                },
                recommendation: Self::generate_trend_recommendation(&trend),
                generated_at: Utc::now(),
            });
        }

        if let Ok(prediction) =
            self.predict_metric(metric_name, self.config.prediction_horizon).await
        {
            insights.push(AnalyticsInsight {
                insight_type: InsightType::Prediction,
                title: format!("Prediction for {metric_name}"),
                description: format!(
                    "Predicted value: {:.2} (confidence: {:.1}%)",
                    prediction.predicted_value,
                    prediction.confidence * 100.0
                ),
                severity: AnomalySeverity::Low,
                recommendation: "Monitor predicted values for planning".to_string(),
                generated_at: Utc::now(),
            });
        }

        info!("Generated {} insights for metric: {}", insights.len(), metric_name);
        Ok(insights)
    }

    fn generate_trend_recommendation(trend: &TrendAnalysis) -> String {
        match trend.direction {
            TrendDirection::Increasing => {
                if trend.strength > 0.8 {
                    "Strong upward trend detected. Consider scaling resources or investigating cause."
                        .to_string()
                } else {
                    "Gradual increase observed. Monitor for continued growth.".to_string()
                }
            }
            TrendDirection::Decreasing => {
                if trend.strength > 0.8 {
                    "Strong downward trend detected. Investigate potential issues or optimization opportunities."
                        .to_string()
                } else {
                    "Gradual decrease observed. Monitor for continued decline.".to_string()
                }
            }
            TrendDirection::Stable => {
                "Metric remains stable. No immediate action required.".to_string()
            }
            TrendDirection::Volatile => {
                "High volatility detected. Consider investigating root causes.".to_string()
            }
        }
    }

    /// Get analytics statistics
    pub async fn get_analytics_statistics(&self) -> AnalyticsStatistics {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Start background analytics processing
    ///
    /// # Errors
    ///
    /// This function currently always returns `Ok(())`; errors from the background task are logged.
    pub fn start_background_processing(&self) -> SongbirdResult<()> {
        info!("Starting background analytics processing");

        let engine = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                if let Err(e) = engine.retrain_all_models().await {
                    error!("Model retraining failed: {e}");
                }

                if let Err(e) = engine.cleanup_old_data().await {
                    error!("Data cleanup failed: {e}");
                }
            }
        });

        info!("Background analytics processing started");
        Ok(())
    }

    async fn retrain_all_models(&self) -> SongbirdResult<()> {
        let series_map = self.time_series.read().await;
        let metric_names: Vec<String> = series_map.keys().cloned().collect();
        drop(series_map);

        for metric_name in metric_names {
            if let Err(e) = self.train_anomaly_model(&metric_name).await {
                warn!("Failed to retrain model for {metric_name}: {e}");
            }
        }

        debug!("Model retraining cycle completed");
        Ok(())
    }

    async fn cleanup_old_data(&self) -> SongbirdResult<()> {
        let mut series_map = self.time_series.write().await;
        let cutoff_time = Utc::now() - chrono::Duration::hours(24);

        for series in series_map.values_mut() {
            series.data_points.retain(|point| point.timestamp > cutoff_time);
        }

        debug!("Old data cleanup completed");
        Ok(())
    }
}

impl Clone for ProductionAnalyticsEngine {
    fn clone(&self) -> Self {
        Self {
            time_series: Arc::clone(&self.time_series),
            trend_cache: Arc::clone(&self.trend_cache),
            anomaly_models: Arc::clone(&self.anomaly_models),
            prediction_models: Arc::clone(&self.prediction_models),
            config: self.config.clone(),
            stats: Arc::clone(&self.stats),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    fn roundtrip_json<T>(v: &T)
    where
        T: Serialize + for<'de> Deserialize<'de>,
    {
        let a = serde_json::to_value(v).expect("serialize");
        let back: T = serde_json::from_value(a.clone()).expect("deserialize");
        let b = serde_json::to_value(&back).expect("serialize again");
        assert_eq!(a, b);
    }

    fn roundtrip_json_ne<T>(v: &T)
    where
        T: Serialize + for<'de> Deserialize<'de>,
    {
        let a = serde_json::to_value(v).expect("serialize");
        let back: T = serde_json::from_value(a.clone()).expect("deserialize");
        let b = serde_json::to_value(&back).expect("serialize again");
        assert_eq!(a, b);
    }

    #[test]
    fn analytics_config_default() {
        let c = AnalyticsConfig::default();
        assert_eq!(c.max_data_points, 10000);
        assert!((c.anomaly_sensitivity - 2.0).abs() < f64::EPSILON);
        assert_eq!(c.prediction_horizon, Duration::from_secs(3600));
        assert_eq!(c.retraining_interval, Duration::from_secs(86400));
        assert!(c.enable_real_time);
    }

    #[test]
    fn analytics_statistics_default() {
        let s = AnalyticsStatistics::default();
        assert_eq!(s.total_data_points, 0);
        assert_eq!(s.trends_analyzed, 0);
        assert_eq!(s.anomalies_detected, 0);
        assert_eq!(s.predictions_made, 0);
        assert_eq!(s.models_trained, 0);
        assert_eq!(s.avg_processing_time, Duration::ZERO);
    }

    #[test]
    fn serde_roundtrip_data_point() {
        let dp = DataPoint {
            timestamp: Utc::now(),
            metric_name: "cpu".to_string(),
            value: 42.5,
            metadata: HashMap::from([("host".to_string(), "a".to_string())]),
        };
        roundtrip_json(&dp);
    }

    #[test]
    fn serde_roundtrip_trend_analysis() {
        let t = TrendAnalysis {
            direction: TrendDirection::Increasing,
            strength: 0.75,
            duration: Duration::from_secs(120),
            confidence: 0.9,
            analyzed_at: Utc::now(),
        };
        roundtrip_json(&t);
    }

    #[test]
    fn serde_roundtrip_anomaly_result() {
        let a = AnomalyResult {
            anomaly_type: AnomalyType::Spike,
            severity: AnomalySeverity::High,
            score: 3.1,
            expected_value: 10.0,
            actual_value: 20.0,
            detected_at: Utc::now(),
            description: "spike".to_string(),
        };
        roundtrip_json(&a);
    }

    #[test]
    fn serde_roundtrip_prediction_result() {
        let p = PredictionResult {
            metric_name: "m".to_string(),
            predicted_value: 1.0,
            confidence: 0.8,
            horizon: Duration::from_secs(600),
            predicted_at: Utc::now(),
            method: PredictionMethod::LinearRegression,
        };
        roundtrip_json(&p);
    }

    #[test]
    fn serde_roundtrip_analytics_config() {
        let c = AnalyticsConfig::default();
        roundtrip_json(&c);
    }

    #[test]
    fn serde_roundtrip_anomaly_model() {
        let m = AnomalyModel {
            name: "x".to_string(),
            baseline_mean: 5.0,
            baseline_std: 0.5,
            threshold_multiplier: 2.0,
            training_size: 100,
            last_updated: Utc::now(),
        };
        roundtrip_json(&m);
    }

    #[test]
    fn serde_roundtrip_prediction_model() {
        let m = PredictionModel {
            name: "p".to_string(),
            coefficients: vec![1.0, 2.0],
            accuracy: 0.95,
            training_size: 50,
            last_trained: Utc::now(),
        };
        roundtrip_json(&m);
    }

    #[test]
    fn serde_roundtrip_analytics_insight() {
        let i = AnalyticsInsight {
            insight_type: InsightType::Trend,
            title: "t".to_string(),
            description: "d".to_string(),
            severity: AnomalySeverity::Medium,
            recommendation: "r".to_string(),
            generated_at: Utc::now(),
        };
        roundtrip_json(&i);
    }

    #[test]
    fn serde_roundtrip_trend_direction_variants() {
        for d in [
            TrendDirection::Increasing,
            TrendDirection::Decreasing,
            TrendDirection::Stable,
            TrendDirection::Volatile,
        ] {
            roundtrip_json_ne(&d);
        }
    }

    #[test]
    fn serde_roundtrip_enums() {
        roundtrip_json_ne(&AnomalyType::Oscillation);
        roundtrip_json_ne(&AnomalySeverity::Critical);
        roundtrip_json_ne(&PredictionMethod::NeuralNetwork);
        roundtrip_json_ne(&InsightType::Resource);
    }

    #[test]
    #[expect(
        clippy::cast_precision_loss,
        reason = "Test fixture: loop index cast to f64 for synthetic series values"
    )]
    fn linear_regression_straight_line_positive_slope() {
        let base = Utc::now();
        let points: Vec<DataPoint> = (0..5)
            .map(|i| DataPoint {
                timestamp: base + chrono::Duration::seconds(i),
                metric_name: "m".to_string(),
                value: i as f64,
                metadata: HashMap::new(),
            })
            .collect();
        let refs: Vec<&DataPoint> = points.iter().collect();
        let (slope, r2) = ProductionAnalyticsEngine::calculate_linear_regression(&refs);
        assert!(slope > 0.9 && slope < 1.1, "slope={slope}");
        assert!(r2 > 0.99, "r2={r2}");
    }

    #[test]
    fn linear_regression_insufficient_points() {
        let p = DataPoint {
            timestamp: Utc::now(),
            metric_name: "m".to_string(),
            value: 1.0,
            metadata: HashMap::new(),
        };
        let (slope, r2) = ProductionAnalyticsEngine::calculate_linear_regression(&[&p]);
        assert_eq!((slope, r2), (0.0, 0.0));
    }

    #[tokio::test]
    async fn engine_add_point_and_stats() {
        let engine = ProductionAnalyticsEngine::new(AnalyticsConfig {
            enable_real_time: false,
            ..AnalyticsConfig::default()
        });
        let dp = DataPoint {
            timestamp: Utc::now(),
            metric_name: "latency".to_string(),
            value: 10.0,
            metadata: HashMap::new(),
        };
        engine.add_data_point(dp).await.expect("add");
        let s = engine.get_analytics_statistics().await;
        assert_eq!(s.total_data_points, 1);
    }

    #[tokio::test]
    async fn detect_anomaly_creates_model() {
        let engine = ProductionAnalyticsEngine::new(AnalyticsConfig::default());
        let r = engine.detect_anomaly("m", 100.0).await.expect("detect");
        assert_eq!(r.actual_value, 100.0);
        assert!(r.score >= 0.0);
    }

    #[tokio::test]
    async fn trend_analysis_requires_data() {
        let engine = ProductionAnalyticsEngine::new(AnalyticsConfig::default());
        let err = engine.analyze_trends("missing", 5).await.expect_err("err");
        assert!(err.to_string().contains("Unknown"));
    }

    #[test]
    fn generate_trend_recommendation_branches() {
        let hi = TrendAnalysis {
            direction: TrendDirection::Increasing,
            strength: 0.9,
            duration: Duration::from_secs(1),
            confidence: 0.5,
            analyzed_at: Utc::now(),
        };
        assert!(
            ProductionAnalyticsEngine::generate_trend_recommendation(&hi).contains("Strong upward")
        );
        let lo = TrendAnalysis {
            direction: TrendDirection::Increasing,
            strength: 0.5,
            duration: Duration::from_secs(1),
            confidence: 0.5,
            analyzed_at: Utc::now(),
        };
        assert!(
            ProductionAnalyticsEngine::generate_trend_recommendation(&lo)
                .contains("Gradual increase")
        );
        let st = TrendAnalysis {
            direction: TrendDirection::Stable,
            strength: 0.1,
            duration: Duration::from_secs(1),
            confidence: 0.5,
            analyzed_at: Utc::now(),
        };
        assert!(ProductionAnalyticsEngine::generate_trend_recommendation(&st).contains("stable"));
        let vo = TrendAnalysis {
            direction: TrendDirection::Volatile,
            strength: 0.9,
            duration: Duration::from_secs(1),
            confidence: 0.5,
            analyzed_at: Utc::now(),
        };
        assert!(
            ProductionAnalyticsEngine::generate_trend_recommendation(&vo).contains("volatility")
        );
    }
}
