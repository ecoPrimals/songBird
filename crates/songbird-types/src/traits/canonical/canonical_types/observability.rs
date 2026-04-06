// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Tracing span context and metric query types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Correlate distributed traces across services.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanContext {
    /// Root trace id.
    pub trace_id: String,
    /// This span id.
    pub span_id: String,
    /// Parent span when not a root span.
    pub parent_span_id: Option<String>,
    /// Cross-service key/value baggage.
    pub baggage: HashMap<String, String>,
}

/// Select a time range and labels when querying metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricQuery {
    /// Metric series name.
    pub metric_name: String,
    /// Inclusive range start.
    pub start_time: SystemTime,
    /// Inclusive range end.
    pub end_time: SystemTime,
    /// Label filters (job, instance, etc.).
    pub labels: HashMap<String, String>,
    /// Optional aggregation function name.
    pub aggregation: Option<String>,
}

/// One sampled metric point returned to callers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricResult {
    /// Metric series name.
    pub metric_name: String,
    /// Sample timestamp.
    pub timestamp: SystemTime,
    /// Observed value.
    pub value: f64,
    /// Labels attached to this sample.
    pub labels: HashMap<String, String>,
}
