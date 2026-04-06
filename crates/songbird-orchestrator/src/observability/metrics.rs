// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Metrics helpers

use super::{MetricType, MetricValue};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

/// Helper to create counter metric
pub fn counter(name: impl Into<Arc<str>>, value: f64) -> MetricValue {
    MetricValue {
        name: name.into(),
        metric_type: MetricType::Counter,
        value,
        timestamp: Utc::now(),
        labels: HashMap::new(),
    }
}

/// Helper to create gauge metric
pub fn gauge(name: impl Into<Arc<str>>, value: f64) -> MetricValue {
    MetricValue {
        name: name.into(),
        metric_type: MetricType::Gauge,
        value,
        timestamp: Utc::now(),
        labels: HashMap::new(),
    }
}

/// Helper to create histogram metric
pub fn histogram(name: impl Into<Arc<str>>, value: f64) -> MetricValue {
    MetricValue {
        name: name.into(),
        metric_type: MetricType::Histogram,
        value,
        timestamp: Utc::now(),
        labels: HashMap::new(),
    }
}
