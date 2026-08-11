// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::{
    AnomalyModel, AnomalyResult, AnomalySeverity, AnomalyType, DataPoint,
    ProductionAnalyticsEngine, TimeSeries,
};
use chrono::Utc;
use songbird_types::SongbirdResult;
use std::collections::VecDeque;
use tracing::{debug, info};

impl ProductionAnalyticsEngine {
    /// Add data point for analysis
    ///
    /// # Errors
    ///
    /// Returns an error if internal storage or real-time anomaly checks fail.
    pub async fn add_data_point(&self, data_point: DataPoint) -> SongbirdResult<()> {
        let metric_name = data_point.metric_name.clone();
        let check_real_time = self.config.enable_real_time;

        {
            let mut series_map = self.time_series.write().unwrap_or_else(std::sync::PoisonError::into_inner);
            let series = series_map.entry(metric_name.clone()).or_insert_with(|| TimeSeries {
                name: metric_name.clone(),
                data_points: VecDeque::new(),
                max_points: self.config.max_data_points,
            });

            series.data_points.push_back(data_point.clone());

            if series.data_points.len() > series.max_points {
                series.data_points.pop_front();
            }
        }

        self.stats.write().unwrap_or_else(std::sync::PoisonError::into_inner).total_data_points += 1;

        if check_real_time
            && let Ok(anomaly) = self.detect_anomaly(&metric_name, data_point.value).await
            && anomaly.severity != AnomalySeverity::Low
        {
            info!("Anomaly detected: {:?} - {}", anomaly.anomaly_type, anomaly.description);
        }

        debug!("Added data point for metric: {} (value: {})", metric_name, data_point.value);
        Ok(())
    }

    /// Detect anomalies in real-time
    ///
    /// # Errors
    ///
    /// Returns an error if concurrent access to internal state fails.
    #[expect(clippy::unused_async, reason = "async for API stability; future I/O-backed models")]
    pub async fn detect_anomaly(
        &self,
        metric_name: &str,
        value: f64,
    ) -> SongbirdResult<AnomalyResult> {
        let model = {
            let models = self.anomaly_models.read().unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(model) = models.get(metric_name) {
                model.clone()
            } else {
                drop(models);
                let new_model = self.create_anomaly_model(metric_name, value);
                let mut models = self.anomaly_models.write().unwrap_or_else(std::sync::PoisonError::into_inner);
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
            let mut stats = self.stats.write().unwrap_or_else(std::sync::PoisonError::into_inner);
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
}
