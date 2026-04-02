// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use super::{AnalyticsStatistics, ProductionAnalyticsEngine};
use chrono::Utc;
use songbird_types::SongbirdResult;
use std::time::Duration;
use tracing::{debug, error, info, warn};

impl ProductionAnalyticsEngine {
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
