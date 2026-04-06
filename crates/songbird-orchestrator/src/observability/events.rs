// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Event Streaming System
//!
//! Provides real-time event streaming for:
//! - Task lifecycle events
//! - Metrics updates
//! - State changes
//! - WebSocket integration
//!
//! Modern async patterns, zero unsafe code, production-ready.

use crate::task_lifecycle::{TaskId, TaskStatus, UserId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use songbird_types::SongbirdResult;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tracing::debug;

/// Task event type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskEventType {
    /// Task was created
    Created,

    /// Task was queued for execution
    Queued,

    /// Task execution started
    Started,

    /// Progress update
    ProgressUpdate {
        progress: f32,
    },

    /// Status changed
    StatusChanged {
        from: TaskStatus,
        to: TaskStatus,
    },

    /// Tower assignment changed
    TowerChanged {
        from: Option<Arc<str>>,
        to: Arc<str>,
    },

    /// Protocol changed
    ProtocolChanged {
        from: Arc<str>,
        to: Arc<str>,
    },

    /// Checkpoint created
    CheckpointCreated {
        checkpoint_id: Arc<str>,
    },

    /// Error occurred
    Error {
        error: Arc<str>,
        recoverable: bool,
    },

    /// Task completed successfully
    Completed,

    /// Task failed
    Failed {
        error: Arc<str>,
    },

    /// Task was cancelled
    Cancelled {
        reason: Option<Arc<str>>,
    },
}

/// Task event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEvent {
    /// Unique event ID
    pub event_id: Arc<str>,

    /// Task this event relates to
    pub task_id: TaskId,

    /// User who owns the task
    pub user_id: UserId,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Event type with details
    pub event_type: TaskEventType,

    /// Optional additional context
    pub context: HashMap<Arc<str>, Arc<str>>,
}

impl TaskEvent {
    /// Create a new task event
    #[must_use]
    pub fn new(task_id: TaskId, user_id: UserId, event_type: TaskEventType) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string().into(),
            task_id,
            user_id,
            timestamp: Utc::now(),
            event_type,
            context: HashMap::new(),
        }
    }

    /// Add context to the event
    pub fn with_context(mut self, key: impl Into<Arc<str>>, value: impl Into<Arc<str>>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
}

/// Event filter for subscriptions
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    /// Filter by user
    pub user_id: Option<UserId>,

    /// Filter by task
    pub task_id: Option<TaskId>,

    /// Filter by event types
    pub event_types: Vec<TaskEventType>,
}

impl EventFilter {
    /// Create filter for a specific user
    #[must_use]
    pub fn for_user(user_id: UserId) -> Self {
        Self {
            user_id: Some(user_id),
            ..Default::default()
        }
    }

    /// Create filter for a specific task
    #[must_use]
    pub fn for_task(task_id: TaskId) -> Self {
        Self {
            task_id: Some(task_id),
            ..Default::default()
        }
    }

    /// Check if event matches this filter
    #[must_use]
    pub fn matches(&self, event: &TaskEvent) -> bool {
        // Check user filter
        if let Some(ref user_id) = self.user_id
            && &event.user_id != user_id
        {
            return false;
        }

        // Check task filter
        if let Some(task_id) = self.task_id
            && event.task_id != task_id
        {
            return false;
        }

        // Check event type filter
        if !self.event_types.is_empty() && !self.event_types.contains(&event.event_type) {
            return false;
        }

        true
    }
}

/// Event stream manager
pub struct EventStreamManager {
    /// Global event broadcaster
    broadcaster: broadcast::Sender<TaskEvent>,

    /// Event history (recent events only)
    history: Arc<RwLock<Vec<TaskEvent>>>,

    /// Maximum history size
    max_history_size: usize,
}

impl EventStreamManager {
    /// Create a new event stream manager
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(1000, 10000)
    }

    /// Create with specific capacities
    #[must_use]
    pub fn with_capacity(channel_capacity: usize, max_history: usize) -> Self {
        let (tx, _) = broadcast::channel(channel_capacity);

        Self {
            broadcaster: tx,
            history: Arc::new(RwLock::new(Vec::with_capacity(max_history))),
            max_history_size: max_history,
        }
    }

    /// Emit an event to all subscribers
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn emit(&self, event: TaskEvent) -> SongbirdResult<()> {
        debug!("Emitting event: {:?} for task {}", event.event_type, event.task_id);

        // Broadcast to subscribers
        match self.broadcaster.send(event.clone()) {
            Ok(subscriber_count) => {
                debug!("Event broadcasted to {} subscribers", subscriber_count);
            }
            Err(_) => {
                // No subscribers, that's okay
                debug!("Event emitted but no active subscribers");
            }
        }

        // Add to history
        let mut history = self.history.write().await;
        history.push(event);

        // Trim history if needed
        if history.len() > self.max_history_size {
            let excess = history.len() - self.max_history_size;
            history.drain(0..excess);
        }

        Ok(())
    }

    /// Subscribe to all events
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<TaskEvent> {
        self.broadcaster.subscribe()
    }

    /// Subscribe with a filter (returns filtered receiver)
    #[must_use]
    pub fn subscribe_filtered(&self, filter: EventFilter) -> FilteredEventReceiver {
        FilteredEventReceiver {
            receiver: self.broadcaster.subscribe(),
            filter,
        }
    }

    /// Get recent event history
    pub async fn get_history(&self, limit: Option<usize>) -> Vec<TaskEvent> {
        let history = self.history.read().await;

        match limit {
            Some(n) => {
                let start = history.len().saturating_sub(n);
                history[start..].to_vec()
            }
            None => history.clone(),
        }
    }

    /// Get history for a specific task
    pub async fn get_task_history(&self, task_id: TaskId) -> Vec<TaskEvent> {
        let history = self.history.read().await;
        history.iter().filter(|e| e.task_id == task_id).cloned().collect()
    }

    /// Get history for a specific user
    pub async fn get_user_history(&self, user_id: &UserId) -> Vec<TaskEvent> {
        let history = self.history.read().await;
        history.iter().filter(|e| &e.user_id == user_id).cloned().collect()
    }

    /// Clear old history (for memory management)
    pub async fn clear_history_before(&self, before: DateTime<Utc>) -> usize {
        let mut history = self.history.write().await;
        let initial_len = history.len();

        history.retain(|event| event.timestamp >= before);

        let removed = initial_len - history.len();
        if removed > 0 {
            debug!("Cleared {} old events from history", removed);
        }

        removed
    }

    /// Get total event count in history
    pub async fn history_size(&self) -> usize {
        let history = self.history.read().await;
        history.len()
    }
}

impl Default for EventStreamManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Filtered event receiver
pub struct FilteredEventReceiver {
    receiver: broadcast::Receiver<TaskEvent>,
    filter: EventFilter,
}

impl FilteredEventReceiver {
    /// Receive next matching event
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn recv(&mut self) -> Result<TaskEvent, broadcast::error::RecvError> {
        loop {
            let event = self.receiver.recv().await?;

            if self.filter.matches(&event) {
                return Ok(event);
            }
            // Skip non-matching events
        }
    }

    /// Try to receive without blocking
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn try_recv(&mut self) -> Result<TaskEvent, broadcast::error::TryRecvError> {
        loop {
            let event = self.receiver.try_recv()?;

            if self.filter.matches(&event) {
                return Ok(event);
            }
            // Skip non-matching events
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    fn test_event(task_id: TaskId, user_id: UserId) -> TaskEvent {
        TaskEvent::new(task_id, user_id, TaskEventType::Created)
    }

    #[tokio::test]
    async fn test_event_creation() {
        let task_id = TaskId::new();
        let user_id = UserId::from("alice");

        let event = TaskEvent::new(
            task_id,
            user_id.clone(),
            TaskEventType::ProgressUpdate {
                progress: 0.5,
            },
        );

        assert_eq!(event.task_id, task_id);
        assert_eq!(event.user_id, user_id);
    }

    #[tokio::test]
    async fn test_event_with_context() {
        let task_id = TaskId::new();
        let user_id = UserId::from("alice");

        let event = TaskEvent::new(task_id, user_id, TaskEventType::Created)
            .with_context("tower", "tower-a")
            .with_context("protocol", "http");

        assert_eq!(event.context.get("tower").map(std::convert::AsRef::as_ref), Some("tower-a"));
        assert_eq!(event.context.get("protocol").map(std::convert::AsRef::as_ref), Some("http"));
    }

    #[tokio::test]
    async fn test_event_filter() {
        let task_id = TaskId::new();
        let user_id = UserId::from("alice");

        let filter = EventFilter::for_user(user_id.clone());
        let event = test_event(task_id, user_id);

        assert!(filter.matches(&event));

        let other_user = UserId::from("bob");
        let other_event = test_event(task_id, other_user);

        assert!(!filter.matches(&other_event));
    }

    #[tokio::test]
    async fn test_event_emission() {
        let manager = EventStreamManager::new();
        let task_id = TaskId::new();
        let user_id = UserId::from("alice");

        let event = test_event(task_id, user_id);

        let result = manager.emit(event.clone()).await;
        assert!(result.is_ok());

        // Check history
        let history = manager.get_history(None).await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].task_id, task_id);
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let manager = EventStreamManager::new();
        let mut receiver = manager.subscribe();

        let task_id = TaskId::new();
        let user_id = UserId::from("alice");
        let event = test_event(task_id, user_id);

        // Emit event
        manager.emit(event.clone()).await.unwrap();

        // Receive event
        let received =
            tokio::time::timeout(tokio::time::Duration::from_millis(100), receiver.recv())
                .await
                .unwrap()
                .unwrap();

        assert_eq!(received.task_id, task_id);
    }

    #[tokio::test]
    async fn test_filtered_subscription() {
        let manager = EventStreamManager::new();
        let user_id = UserId::from("alice");

        let filter = EventFilter::for_user(user_id.clone());
        let mut receiver = manager.subscribe_filtered(filter);

        // Emit events for different users
        let alice_task = TaskId::new();
        let bob_task = TaskId::new();

        manager.emit(test_event(bob_task, UserId::from("bob"))).await.unwrap();

        manager.emit(test_event(alice_task, user_id.clone())).await.unwrap();

        // Should receive only Alice's event
        let received =
            tokio::time::timeout(tokio::time::Duration::from_millis(100), receiver.recv())
                .await
                .unwrap()
                .unwrap();

        assert_eq!(received.task_id, alice_task);
        assert_eq!(received.user_id, user_id);
    }

    #[tokio::test]
    async fn test_task_history() {
        let manager = EventStreamManager::new();
        let task_id = TaskId::new();
        let user_id = UserId::from("alice");

        // Emit multiple events for same task
        for i in 0..5 {
            let event = TaskEvent::new(
                task_id,
                user_id.clone(),
                TaskEventType::ProgressUpdate {
                    progress: i as f32 / 5.0,
                },
            );
            manager.emit(event).await.unwrap();
        }

        let history = manager.get_task_history(task_id).await;
        assert_eq!(history.len(), 5);
    }

    #[tokio::test]
    async fn test_user_history() {
        let manager = EventStreamManager::new();
        let user_id = UserId::from("alice");

        // Emit events for different tasks by same user
        for _ in 0..3 {
            let task_id = TaskId::new();
            let event = test_event(task_id, user_id.clone());
            manager.emit(event).await.unwrap();
        }

        let history = manager.get_user_history(&user_id).await;
        assert_eq!(history.len(), 3);
    }

    #[tokio::test]
    async fn test_history_cleanup() {
        let manager = EventStreamManager::new();
        let task_id = TaskId::new();
        let user_id = UserId::from("alice");

        // Emit several events
        for _ in 0..10 {
            let event = test_event(task_id, user_id.clone());
            manager.emit(event).await.unwrap();
        }

        assert_eq!(manager.history_size().await, 10);

        // Clear events from the future (should clear all)
        let future = Utc::now() + chrono::Duration::days(1);
        let cleared = manager.clear_history_before(future).await;

        assert_eq!(cleared, 10);
        assert_eq!(manager.history_size().await, 0);
    }

    #[tokio::test]
    async fn test_history_size_limit() {
        let manager = EventStreamManager::with_capacity(100, 5);
        let task_id = TaskId::new();
        let user_id = UserId::from("alice");

        // Emit more events than history limit
        for _ in 0..10 {
            let event = test_event(task_id, user_id.clone());
            manager.emit(event).await.unwrap();
        }

        // Should only keep last 5
        assert_eq!(manager.history_size().await, 5);
    }
}
