// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Resource Usage Tracker
//!
//! Tracks resource usage over time for:
//! - Billing/accounting
//! - Trend analysis
//! - Capacity planning

use super::{ResourceAmount, ResourceType};
use crate::task_lifecycle::{TaskId, UserId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Usage record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub user_id: UserId,
    pub task_id: TaskId,
    pub resource_type: ResourceType,
    pub amount: ResourceAmount,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_seconds: Option<f64>, // Changed to f64 for fractional seconds
}

impl UsageRecord {
    #[must_use]
    pub fn new(
        user_id: UserId,
        task_id: TaskId,
        resource_type: ResourceType,
        amount: ResourceAmount,
    ) -> Self {
        Self {
            user_id,
            task_id,
            resource_type,
            amount,
            start_time: Utc::now(),
            end_time: None,
            duration_seconds: None,
        }
    }

    pub fn complete(&mut self) {
        let end = Utc::now();
        // Use fractional seconds for accurate short-duration tracking
        let duration = (end - self.start_time).num_milliseconds() as f64 / 1000.0;
        self.end_time = Some(end);
        self.duration_seconds = Some(duration);
    }

    /// Calculate resource-seconds (amount * duration)
    #[must_use]
    pub fn resource_seconds(&self) -> f64 {
        if let Some(duration) = self.duration_seconds {
            self.amount.value * duration
        } else {
            0.0
        }
    }
}

/// Usage summary for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    pub user_id: UserId,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_by_resource: HashMap<ResourceType, f64>, // resource-seconds
    pub task_count: usize,
}

/// Usage tracker
pub struct UsageTracker {
    active_records: Arc<RwLock<HashMap<TaskId, Vec<UsageRecord>>>>,
    completed_records: Arc<RwLock<Vec<UsageRecord>>>,
}

impl UsageTracker {
    #[must_use]
    pub fn new() -> Self {
        Self {
            active_records: Arc::new(RwLock::new(HashMap::new())),
            completed_records: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start tracking resource usage for a task
    pub async fn start_tracking(
        &self,
        task_id: TaskId,
        user_id: UserId,
        resources: &HashMap<ResourceType, ResourceAmount>,
    ) {
        let mut active = self.active_records.write().await;

        let records: Vec<UsageRecord> = resources
            .iter()
            .map(|(resource_type, amount)| {
                UsageRecord::new(user_id.clone(), task_id, *resource_type, *amount)
            })
            .collect();

        active.insert(task_id, records);
    }

    /// Stop tracking and finalize records
    pub async fn stop_tracking(&self, task_id: TaskId) {
        let mut active = self.active_records.write().await;

        if let Some(mut records) = active.remove(&task_id) {
            // Complete all records
            for record in &mut records {
                record.complete();
            }

            // Move to completed
            let mut completed = self.completed_records.write().await;
            completed.extend(records);
        }
    }

    /// Get usage summary for a user over a time period
    pub async fn get_summary(
        &self,
        user_id: &UserId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> UsageSummary {
        let completed = self.completed_records.read().await;

        let mut total_by_resource: HashMap<ResourceType, f64> = HashMap::new();
        let mut task_count = 0;
        let mut seen_tasks = std::collections::HashSet::new();

        for record in completed.iter() {
            if record.user_id != *user_id {
                continue;
            }

            // Check if record is in time range
            if record.start_time < start || record.start_time > end {
                continue;
            }

            // Track unique tasks
            if seen_tasks.insert(record.task_id) {
                task_count += 1;
            }

            // Sum resource-seconds
            let resource_seconds = record.resource_seconds();
            *total_by_resource.entry(record.resource_type).or_insert(0.0) += resource_seconds;
        }

        UsageSummary {
            user_id: user_id.clone(),
            period_start: start,
            period_end: end,
            total_by_resource,
            task_count,
        }
    }

    /// Get all active tasks
    pub async fn get_active_tasks(&self) -> Vec<TaskId> {
        let active = self.active_records.read().await;
        active.keys().copied().collect()
    }

    /// Get total completed records
    pub async fn total_completed_records(&self) -> usize {
        let completed = self.completed_records.read().await;
        completed.len()
    }

    /// Cleanup old records (for memory management)
    pub async fn cleanup_old_records(&self, before: DateTime<Utc>) -> usize {
        let mut completed = self.completed_records.write().await;
        let initial_len = completed.len();

        completed.retain(|record| record.start_time >= before);

        initial_len - completed.len()
    }
}

impl Default for UsageTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::ResourceUnit;
    use super::*;

    #[test]
    fn test_usage_record() {
        let mut record = UsageRecord::new(
            UserId::from("alice"),
            TaskId::new(),
            ResourceType::Cpu,
            ResourceAmount::new(2.0, ResourceUnit::Cores),
        );

        assert!(record.end_time.is_none());
        assert!(record.duration_seconds.is_none());

        record.start_time = Utc::now() - chrono::Duration::seconds(1);
        record.complete();

        assert!(record.end_time.is_some());
        assert!(record.duration_seconds.is_some());
        let resource_secs = record.resource_seconds();
        assert!(resource_secs > 0.0, "Expected resource_seconds > 0, got {resource_secs}");
    }

    #[tokio::test]
    async fn test_tracking_lifecycle() {
        let tracker = UsageTracker::new();
        let task_id = TaskId::new();
        let user_id = UserId::from("alice");

        let mut resources = HashMap::new();
        resources.insert(ResourceType::Cpu, ResourceAmount::new(2.0, ResourceUnit::Cores));

        // Start tracking
        tracker.start_tracking(task_id, user_id.clone(), &resources).await;

        let active = tracker.get_active_tasks().await;
        assert_eq!(active.len(), 1);
        assert_eq!(active[0], task_id);

        // Stop tracking
        tracker.stop_tracking(task_id).await;

        let active_after = tracker.get_active_tasks().await;
        assert_eq!(active_after.len(), 0);

        let completed_count = tracker.total_completed_records().await;
        assert_eq!(completed_count, 1);
    }

    #[tokio::test]
    async fn test_usage_summary() {
        let tracker = UsageTracker::new();
        let user_id = UserId::from("alice");

        let start_time = Utc::now() - chrono::Duration::seconds(10);

        for _ in 0..3 {
            let task_id = TaskId::new();
            let mut resources = HashMap::new();
            resources.insert(ResourceType::Cpu, ResourceAmount::new(2.0, ResourceUnit::Cores));

            tracker.start_tracking(task_id, user_id.clone(), &resources).await;

            {
                let mut active = tracker.active_records.write().await;
                if let Some(records) = active.get_mut(&task_id) {
                    for r in records.iter_mut() {
                        r.start_time = Utc::now() - chrono::Duration::seconds(1);
                    }
                }
            }

            tracker.stop_tracking(task_id).await;
        }

        let end_time = Utc::now();

        let summary = tracker.get_summary(&user_id, start_time, end_time).await;

        assert_eq!(summary.task_count, 3);
        assert!(summary.total_by_resource.contains_key(&ResourceType::Cpu));
        let cpu_total = summary.total_by_resource[&ResourceType::Cpu];
        assert!(cpu_total > 0.0, "Expected CPU total > 0, got {cpu_total}");
    }

    #[tokio::test]
    async fn test_cleanup_old_records() {
        let tracker = UsageTracker::new();
        let user_id = UserId::from("alice");

        // Create and complete some tasks
        for _ in 0..5 {
            let task_id = TaskId::new();
            let mut resources = HashMap::new();
            resources.insert(ResourceType::Cpu, ResourceAmount::new(1.0, ResourceUnit::Cores));

            tracker.start_tracking(task_id, user_id.clone(), &resources).await;
            tracker.stop_tracking(task_id).await;
        }

        let initial_count = tracker.total_completed_records().await;
        assert_eq!(initial_count, 5);

        // Cleanup records before future date (should remove all)
        let future = Utc::now() + chrono::Duration::days(1);
        let removed = tracker.cleanup_old_records(future).await;

        assert_eq!(removed, 5);

        let final_count = tracker.total_completed_records().await;
        assert_eq!(final_count, 0);
    }
}
