// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Production Analytics Engine
//!
//! Real data processing and machine learning analytics for trend analysis,
//! anomaly detection, and metric prediction. Type definitions live in
//! [`super::types`].

#![allow(missing_docs)]

pub use super::types::*;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

mod aggregation;
mod collection;
mod dashboard;
mod reporting;

#[cfg(test)]
mod tests;

/// Production analytics engine
pub struct ProductionAnalyticsEngine {
    pub(crate) time_series: Arc<RwLock<HashMap<String, TimeSeries>>>,
    pub(crate) trend_cache: Arc<RwLock<HashMap<String, TrendAnalysis>>>,
    pub(crate) anomaly_models: Arc<RwLock<HashMap<String, AnomalyModel>>>,
    pub(crate) prediction_models: Arc<RwLock<HashMap<String, PredictionModel>>>,
    pub(crate) config: AnalyticsConfig,
    pub(crate) stats: Arc<RwLock<AnalyticsStatistics>>,
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
