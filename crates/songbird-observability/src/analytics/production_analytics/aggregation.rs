// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::{
    AnomalyModel, DataPoint, PredictionMethod, PredictionResult, ProductionAnalyticsEngine,
    TrendAnalysis, TrendDirection,
};
use chrono::Utc;
use songbird_types::{SongbirdError, SongbirdResult};
use std::time::Duration;
use tracing::info;

impl ProductionAnalyticsEngine {
    /// Analyze trends for a metric
    ///
    /// # Errors
    ///
    /// Returns an error when the metric is unknown or there are too few data points for the window.
    #[expect(clippy::unused_async, reason = "async for API stability")]
    pub async fn analyze_trends(
        &self,
        metric_name: &str,
        window_size: usize,
    ) -> SongbirdResult<TrendAnalysis> {
        let series_map = self.time_series.read().unwrap_or_else(std::sync::PoisonError::into_inner);
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

        let mut cache = self.trend_cache.write().unwrap_or_else(std::sync::PoisonError::into_inner);
        cache.insert(metric_name.to_string(), trend_analysis.clone());

        let mut stats = self.stats.write().unwrap_or_else(std::sync::PoisonError::into_inner);
        stats.trends_analyzed += 1;

        info!(
            "Trend analysis complete for {}: {:?} (strength: {:.2})",
            metric_name, direction, strength
        );

        Ok(trend_analysis)
    }

    /// Make predictions for a metric
    ///
    /// # Errors
    ///
    /// Returns an error when the metric is unknown or there is insufficient history for a forecast.
    #[expect(clippy::unused_async, reason = "async for API stability")]
    pub async fn predict_metric(
        &self,
        metric_name: &str,
        horizon: Duration,
    ) -> SongbirdResult<PredictionResult> {
        let series_map = self.time_series.read().unwrap_or_else(std::sync::PoisonError::into_inner);
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

        let mut stats = self.stats.write().unwrap_or_else(std::sync::PoisonError::into_inner);
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
    pub(crate) fn calculate_linear_regression(data_points: &[&DataPoint]) -> (f64, f64) {
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
    #[expect(clippy::unused_async, reason = "async for API stability")]
    pub async fn train_anomaly_model(&self, metric_name: &str) -> SongbirdResult<()> {
        let series_map = self.time_series.read().unwrap_or_else(std::sync::PoisonError::into_inner);
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

        let mut models = self.anomaly_models.write().unwrap_or_else(std::sync::PoisonError::into_inner);
        models.insert(metric_name.to_string(), model);

        let mut stats = self.stats.write().unwrap_or_else(std::sync::PoisonError::into_inner);
        stats.models_trained += 1;

        info!("Trained anomaly model for {}: mean={:.2}, std={:.2}", metric_name, mean, std_dev);

        Ok(())
    }
}
