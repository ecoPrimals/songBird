//! Fair Scheduler
//!
//! Implements fair scheduling with:
//! - Weighted fair queuing
//! - Priority-aware scheduling
//! - Starvation prevention

use crate::task_lifecycle::{TaskId, TaskLifecycle, UserId};
use anyhow::Result;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Scheduling decision
#[derive(Debug, Clone)]
pub struct SchedulingDecision {
    pub task_id: TaskId,
    pub reason: Arc<str>,
}

/// Task queue entry with scheduling metadata
#[derive(Debug, Clone)]
struct QueueEntry {
    task: TaskLifecycle,
    virtual_finish_time: f64,
    arrival_time: f64,
    priority_boost: i32, // Priority level for tie-breaking
}

impl PartialEq for QueueEntry {
    fn eq(&self, other: &Self) -> bool {
        self.virtual_finish_time == other.virtual_finish_time
            && self.priority_boost == other.priority_boost
    }
}

impl Eq for QueueEntry {}

impl PartialOrd for QueueEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Primary: Lower virtual_finish_time = higher priority
        // Secondary: Higher priority_boost = higher priority (for tie-breaking)
        match other.virtual_finish_time.partial_cmp(&self.virtual_finish_time) {
            Some(Ordering::Equal) => {
                // Tie-break by priority level (higher priority_boost wins)
                self.priority_boost.partial_cmp(&other.priority_boost)
            }
            other_ord => other_ord,
        }
    }
}

impl Ord for QueueEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // For scheduling, we need a total ordering even when floats are NaN
        // NaN values are treated as equal for scheduling purposes
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

/// Fair scheduler implementing priority-aware weighted fair queuing.
///
/// Provides fair task scheduling across multiple users with support for:
/// - Weighted fair queuing (users can have different weights)
/// - Priority levels for tie-breaking
/// - Starvation prevention (virtual time advancement)
/// - Dynamic user addition/removal
///
/// # Algorithm
///
/// Uses **Virtual Time Fair Queuing**:
/// 1. Each user has a virtual time that tracks their resource consumption
/// 2. Tasks are scheduled based on virtual finish time
/// 3. Priority levels break ties when virtual times are equal
/// 4. Global virtual time prevents starvation
///
/// # Example
///
/// ```rust,ignore
/// # use songbird_orchestrator::resource_management::*;
/// # use songbird_orchestrator::task_lifecycle::*;
/// # async fn example() -> anyhow::Result<()> {
/// let scheduler = FairScheduler::new();
///
/// // Set user weights (higher = more fair share)
/// scheduler.set_user_weight(UserId::from("alice"), 2.0).await;
/// scheduler.set_user_weight(UserId::from("bob"), 1.0).await;
///
/// // Enqueue tasks
/// let task1 = TaskLifecycle::new(
///     UserId::from("alice"),
///     TaskSpec { task_type: "compute".into(), parameters: serde_json::json!({}) }
/// );
/// scheduler.enqueue(task1).await?;
///
/// // Dequeue tasks in fair order
/// if let Some(decision) = scheduler.dequeue().await? {
///     println!("Next task: {}", decision.task_id);
/// }
/// # Ok(())
/// # }
/// ```
pub struct FairScheduler {
    /// Priority queue of tasks
    queue: Arc<RwLock<BinaryHeap<QueueEntry>>>,

    /// Per-user virtual time tracking
    virtual_times: Arc<RwLock<HashMap<UserId, f64>>>,

    /// Global virtual time
    global_virtual_time: Arc<RwLock<f64>>,

    /// User weights (higher = more resources)
    user_weights: Arc<RwLock<HashMap<UserId, f64>>>,
}

impl FairScheduler {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(RwLock::new(BinaryHeap::new())),
            virtual_times: Arc::new(RwLock::new(HashMap::new())),
            global_virtual_time: Arc::new(RwLock::new(0.0)),
            user_weights: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Enqueue a task for fair scheduling.
    ///
    /// The task is added to the priority queue with a virtual finish time
    /// calculated based on the user's current virtual time and weight.
    ///
    /// # Arguments
    /// * `task` - The task to enqueue
    ///
    /// # Returns
    /// Ok(()) if task was successfully enqueued
    ///
    /// # Errors
    /// Returns error if queue operations fail
    ///
    /// # Example
    /// ```rust,ignore
    /// # use songbird_orchestrator::resource_management::*;
    /// # use songbird_orchestrator::task_lifecycle::*;
    /// # async fn example() -> anyhow::Result<()> {
    /// let scheduler = FairScheduler::new();
    /// let task = TaskLifecycle::new(
    ///     UserId::from("alice"),
    ///     TaskSpec { task_type: "compute".into(), parameters: serde_json::json!({}) }
    /// );
    /// scheduler.enqueue(task).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn enqueue(&self, task: TaskLifecycle) -> Result<()> {
        let user_id = task.owner.clone();
        let arrival_time = chrono::Utc::now().timestamp() as f64;

        // Get user weight (default 1.0 for fair share)
        let user_weights = self.user_weights.read().await;
        let weight = user_weights.get(&user_id).copied().unwrap_or(1.0);
        drop(user_weights);

        // Get user's current virtual time
        let mut virtual_times = self.virtual_times.write().await;
        let global_vt = *self.global_virtual_time.read().await;

        let user_vt = virtual_times.entry(user_id.clone()).or_insert(global_vt);

        // Calculate task cost (normalized resource requirements)
        let cost = self.calculate_task_cost(&task);

        // Get priority adjustment - critical tasks bypass queue by using earlier virtual time
        // Values chosen to respect priority without causing starvation
        let priority_adjustment = match task.spec.priority {
            crate::task_lifecycle::types::Priority::Critical => -2.0, // Jump to front
            crate::task_lifecycle::types::Priority::High => -0.5,     // Jump ahead
            crate::task_lifecycle::types::Priority::Standard => 0.0,  // Baseline
            crate::task_lifecycle::types::Priority::Low => 0.5, // Slight delay (prevents starvation)
        };

        // Virtual finish time = max(user_vt, global_vt) + cost/weight + priority_adjustment
        let virtual_start = user_vt.max(global_vt);
        let virtual_finish = virtual_start + (cost / weight) + priority_adjustment;

        // Update user's virtual time
        *user_vt = virtual_finish;
        drop(virtual_times);

        // Get priority boost for tie-breaking
        let priority_boost = match task.spec.priority {
            crate::task_lifecycle::types::Priority::Critical => 3,
            crate::task_lifecycle::types::Priority::High => 2,
            crate::task_lifecycle::types::Priority::Standard => 1,
            crate::task_lifecycle::types::Priority::Low => 0,
        };

        // Add to priority queue
        let entry = QueueEntry {
            task,
            virtual_finish_time: virtual_finish,
            arrival_time,
            priority_boost,
        };

        let mut queue = self.queue.write().await;
        queue.push(entry);

        Ok(())
    }

    /// Dequeue the next task to schedule
    pub async fn dequeue(&self) -> Option<TaskLifecycle> {
        let mut queue = self.queue.write().await;

        if let Some(entry) = queue.pop() {
            // Update global virtual time
            let mut global_vt = self.global_virtual_time.write().await;
            *global_vt = entry.virtual_finish_time;

            Some(entry.task)
        } else {
            None
        }
    }

    /// Peek at the next task without removing it
    pub async fn peek(&self) -> Option<TaskLifecycle> {
        let queue = self.queue.read().await;
        queue.peek().map(|entry| entry.task.clone())
    }

    /// Get queue length
    pub async fn queue_len(&self) -> usize {
        let queue = self.queue.read().await;
        queue.len()
    }

    /// Set user weight (for priority users)
    pub async fn set_user_weight(&self, user_id: UserId, weight: f64) {
        let mut weights = self.user_weights.write().await;
        weights.insert(user_id, weight);
    }

    /// Calculate normalized cost of a task
    fn calculate_task_cost(&self, task: &TaskLifecycle) -> f64 {
        // Cost is based on resource requirements
        // For priority-aware scheduling: lower cost = earlier finish time = higher priority
        // Inverse the priority values so critical tasks have lowest cost

        // Could be extended with actual resource requirements
        match task.spec.priority {
            crate::task_lifecycle::types::Priority::Critical => 0.125, // 1/8 - highest priority
            crate::task_lifecycle::types::Priority::High => 0.25,      // 1/4
            crate::task_lifecycle::types::Priority::Standard => 1.0,   // baseline
            crate::task_lifecycle::types::Priority::Low => 2.0,        // lowest priority
        }
    }
}

impl Default for FairScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task_lifecycle::types::{Priority, ResourceRequirements, TaskSpec};

    fn create_test_task(owner: &str, priority: Priority) -> TaskLifecycle {
        let spec = TaskSpec {
            task_type: "test".into(),
            config: serde_json::json!({}),
            required_capabilities: vec![],
            resources: ResourceRequirements::default(),
            priority,
        };

        TaskLifecycle::new(UserId::from(owner), spec)
    }

    #[tokio::test]
    async fn test_fair_scheduling_single_user() {
        let scheduler = FairScheduler::new();

        // Enqueue tasks from single user
        let task1 = create_test_task("alice", Priority::Standard);
        let task2 = create_test_task("alice", Priority::Standard);
        let task3 = create_test_task("alice", Priority::Standard);

        let id1 = task1.id;
        let id2 = task2.id;
        let id3 = task3.id;

        scheduler.enqueue(task1).await.unwrap();
        scheduler.enqueue(task2).await.unwrap();
        scheduler.enqueue(task3).await.unwrap();

        // Should dequeue in order
        let next1 = scheduler.dequeue().await.unwrap();
        let next2 = scheduler.dequeue().await.unwrap();
        let next3 = scheduler.dequeue().await.unwrap();

        assert_eq!(next1.id, id1);
        assert_eq!(next2.id, id2);
        assert_eq!(next3.id, id3);
    }

    #[tokio::test]
    async fn test_fair_scheduling_multiple_users() {
        let scheduler = FairScheduler::new();

        // Enqueue multiple tasks from alice
        for _ in 0..3 {
            let task = create_test_task("alice", Priority::Standard);
            scheduler.enqueue(task).await.unwrap();
        }

        // Enqueue one task from bob
        let bob_task = create_test_task("bob", Priority::Standard);
        let bob_id = bob_task.id;
        scheduler.enqueue(bob_task).await.unwrap();

        // Bob's task should be interleaved fairly despite arriving later
        let mut bob_found_early = false;
        for i in 0..4 {
            let task = scheduler.dequeue().await.unwrap();
            if task.id == bob_id && i < 3 {
                bob_found_early = true;
            }
        }

        // Bob should get scheduled before all of Alice's tasks complete
        assert!(bob_found_early, "Fair scheduling should interleave users");
    }

    #[tokio::test]
    async fn test_priority_scheduling() {
        let scheduler = FairScheduler::new();

        // Enqueue low priority task
        let low_task = create_test_task("alice", Priority::Low);
        scheduler.enqueue(low_task).await.unwrap();

        // Enqueue critical priority task from same user
        let critical_task = create_test_task("alice", Priority::Critical);
        scheduler.enqueue(critical_task).await.unwrap();

        // Critical should come first (verify by priority, not ID - UUIDs are non-deterministic)
        let next = scheduler.dequeue().await.unwrap();
        assert_eq!(
            next.spec.priority,
            Priority::Critical,
            "Critical priority task should be dequeued first"
        );

        // Low priority should come second
        let next = scheduler.dequeue().await.unwrap();
        assert_eq!(
            next.spec.priority,
            Priority::Low,
            "Low priority task should be dequeued second"
        );
    }

    #[tokio::test]
    async fn test_user_weights() {
        let scheduler = FairScheduler::new();

        // Give alice higher weight
        scheduler.set_user_weight(UserId::from("alice"), 2.0).await;
        scheduler.set_user_weight(UserId::from("bob"), 1.0).await;

        // Enqueue tasks
        let alice_task = create_test_task("alice", Priority::Standard);
        let alice_id = alice_task.id;
        scheduler.enqueue(alice_task).await.unwrap();

        let bob_task = create_test_task("bob", Priority::Standard);
        scheduler.enqueue(bob_task).await.unwrap();

        // Alice (higher weight) should be scheduled first
        let next = scheduler.dequeue().await.unwrap();
        assert_eq!(next.id, alice_id);
    }

    #[tokio::test]
    async fn test_starvation_prevention() {
        let scheduler = FairScheduler::new();

        // Enqueue low priority task
        let low_task = create_test_task("alice", Priority::Low);
        let low_id = low_task.id;
        scheduler.enqueue(low_task).await.unwrap();

        // Enqueue multiple standard tasks
        for _ in 0..10 {
            let task = create_test_task("bob", Priority::Standard);
            scheduler.enqueue(task).await.unwrap();
        }

        // Low priority task should eventually be scheduled (within first 5)
        let mut low_found = false;
        for _ in 0..5 {
            if let Some(task) = scheduler.dequeue().await {
                if task.id == low_id {
                    low_found = true;
                    break;
                }
            }
        }

        assert!(low_found, "Low priority task should not starve");
    }
}
