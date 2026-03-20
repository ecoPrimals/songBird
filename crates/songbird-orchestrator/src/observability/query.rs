// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Query API for metrics

use super::MetricValue;
use chrono::{DateTime, Utc};

/// Query filter
#[derive(Debug, Clone)]
pub struct MetricQuery {
    pub metric_name: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

impl MetricQuery {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            metric_name: None,
            start_time: None,
            end_time: None,
        }
    }

    #[must_use]
    pub fn matches(&self, metric: &MetricValue) -> bool {
        if let Some(ref name) = self.metric_name
            && metric.name.as_ref() != name
        {
            return false;
        }

        if let Some(start) = self.start_time
            && metric.timestamp < start
        {
            return false;
        }

        if let Some(end) = self.end_time
            && metric.timestamp > end
        {
            return false;
        }

        true
    }
}

impl Default for MetricQuery {
    fn default() -> Self {
        Self::new()
    }
}
