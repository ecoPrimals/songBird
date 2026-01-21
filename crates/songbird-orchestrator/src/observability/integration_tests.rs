//! Integration tests for observability features
//!
//! Tests the full observability workflow including:
//! - Event streaming
//! - Event filtering
//! - Multiple subscribers
//! - Event history

#[cfg(test)]
mod tests {
    use crate::observability::{EventFilter, EventStreamManager, TaskEventType};
    use crate::task_lifecycle::{TaskId, UserId};
    use std::sync::Arc;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_event_stream_integration() {
        let manager = Arc::new(EventStreamManager::new());

        // Subscribe to all events
        let mut receiver = manager.subscribe_filtered(EventFilter::default());

        // Emit an event
        let task_id = TaskId::new();
        let user_id = UserId::new("alice");
        let event =
            crate::observability::TaskEvent::new(task_id, user_id.clone(), TaskEventType::Started);

        manager.emit(event.clone()).await.ok();

        // Receive the event
        let received = timeout(Duration::from_secs(1), receiver.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Failed to receive event");

        assert_eq!(received.task_id, task_id);
        assert_eq!(received.user_id, user_id);
        assert!(matches!(received.event_type, TaskEventType::Started));
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let manager = Arc::new(EventStreamManager::new());

        // Create multiple subscribers
        let mut sub1 = manager.subscribe_filtered(EventFilter::default());
        let mut sub2 = manager.subscribe_filtered(EventFilter::default());
        let mut sub3 = manager.subscribe_filtered(EventFilter::default());

        // Emit an event
        let task_id = TaskId::new();
        let user_id = UserId::new("bob");
        let event =
            crate::observability::TaskEvent::new(task_id, user_id, TaskEventType::Completed);

        manager.emit(event).await.ok();

        // All subscribers should receive the event
        let event1 = timeout(Duration::from_secs(1), sub1.recv())
            .await
            .expect("Sub1 timeout")
            .expect("Sub1 failed");
        let event2 = timeout(Duration::from_secs(1), sub2.recv())
            .await
            .expect("Sub2 timeout")
            .expect("Sub2 failed");
        let event3 = timeout(Duration::from_secs(1), sub3.recv())
            .await
            .expect("Sub3 timeout")
            .expect("Sub3 failed");

        assert_eq!(event1.task_id, event2.task_id);
        assert_eq!(event2.task_id, event3.task_id);
    }

    #[tokio::test]
    async fn test_filtered_subscription() {
        let manager = Arc::new(EventStreamManager::new());

        let alice_id = UserId::new("alice");
        let bob_id = UserId::new("bob");

        // Subscribe only to alice's events
        let filter = EventFilter {
            user_id: Some(alice_id.clone()),
            ..Default::default()
        };
        let mut receiver = manager.subscribe_filtered(filter);

        // Emit bob's event (should be filtered out)
        let bob_task = TaskId::new();
        let bob_event =
            crate::observability::TaskEvent::new(bob_task, bob_id, TaskEventType::Started);
        manager.emit(bob_event).await.ok();

        // Emit alice's event (should be received)
        let alice_task = TaskId::new();
        let alice_event = crate::observability::TaskEvent::new(
            alice_task,
            alice_id.clone(),
            TaskEventType::Started,
        );
        manager.emit(alice_event).await.ok();

        // Should only receive alice's event
        let received = timeout(Duration::from_secs(1), receiver.recv())
            .await
            .expect("Timeout")
            .expect("Failed to receive");

        assert_eq!(received.user_id, alice_id);
        assert_eq!(received.task_id, alice_task);
    }

    #[tokio::test]
    async fn test_event_history() {
        let manager = Arc::new(EventStreamManager::new());

        // Emit multiple events
        for i in 0..5 {
            let event = crate::observability::TaskEvent::new(
                TaskId::new(),
                UserId::new(format!("user-{}", i)),
                TaskEventType::Started,
            );
            manager.emit(event).await.ok();
        }

        // Get history
        let history = manager.get_history(Some(10)).await;

        // Should have all 5 events
        assert_eq!(history.len(), 5);

        // Events should be in chronological order (newest first)
        for i in 0..history.len() - 1 {
            assert!(history[i].timestamp >= history[i + 1].timestamp);
        }
    }

    #[tokio::test]
    async fn test_event_lifecycle_workflow() {
        let manager = Arc::new(EventStreamManager::new());
        let mut receiver = manager.subscribe_filtered(EventFilter::default());

        let task_id = TaskId::new();
        let user_id = UserId::new("charlie");

        // Complete lifecycle: Started → Completed
        let events = vec![TaskEventType::Started, TaskEventType::Completed];

        for event_type in events {
            let event = crate::observability::TaskEvent::new(task_id, user_id.clone(), event_type);
            manager.emit(event).await.ok();
        }

        // Receive started event
        let started = timeout(Duration::from_secs(1), receiver.recv())
            .await
            .expect("Timeout on started")
            .expect("Failed to receive started");
        assert!(matches!(started.event_type, TaskEventType::Started));

        // Receive completed event
        let completed = timeout(Duration::from_secs(1), receiver.recv())
            .await
            .expect("Timeout on completed")
            .expect("Failed to receive completed");
        assert!(matches!(completed.event_type, TaskEventType::Completed));
    }

    #[tokio::test]
    async fn test_concurrent_subscribers_and_emitters() {
        let manager = Arc::new(EventStreamManager::new());

        // Create subscribers and ready notifiers
        let (ready_tx, mut ready_rx) = tokio::sync::mpsc::channel(3);
        
        // Spawn multiple subscriber tasks
        let mut handles = vec![];
        for i in 0..3 {
            let manager_clone = Arc::clone(&manager);
            let ready_tx_clone = ready_tx.clone();
            let handle = tokio::spawn(async move {
                let mut receiver = manager_clone.subscribe_filtered(EventFilter::default());
                
                // Signal that this subscriber is ready
                ready_tx_clone.send(()).await.ok();
                
                let mut count = 0;
                while count < 10 {
                    match timeout(Duration::from_secs(2), receiver.recv()).await {
                        Ok(Ok(_)) => count += 1,
                        Ok(Err(_)) => break, // Channel closed
                        Err(_) => break, // Timeout
                    }
                }

                (i, count)
            });
            handles.push(handle);
        }
        
        drop(ready_tx); // Drop original sender

        // Wait for all subscribers to be ready
        let mut ready_count = 0;
        while ready_count < 3 {
            if timeout(Duration::from_millis(100), ready_rx.recv()).await.is_ok() {
                ready_count += 1;
            } else {
                break;
            }
        }

        // Emit 10 events (now that subscribers are ready)
        for j in 0..10 {
            let event = crate::observability::TaskEvent::new(
                TaskId::new(),
                UserId::new(format!("user-{}", j)),
                TaskEventType::Started,
            );
            manager.emit(event).await.ok();
            // Small yield to ensure fair event distribution
            tokio::task::yield_now().await;
        }

        // Wait for all subscribers to receive events
        for handle in handles {
            let (sub_id, count) = handle.await.expect("Subscriber task failed");
            assert_eq!(count, 10, "Subscriber {} didn't receive all events", sub_id);
        }
    }

    #[tokio::test]
    async fn test_task_specific_filtering() {
        let manager = Arc::new(EventStreamManager::new());

        let target_task = TaskId::new();
        let other_task = TaskId::new();

        // Subscribe only to target task
        let filter = EventFilter {
            task_id: Some(target_task),
            ..Default::default()
        };
        let mut receiver = manager.subscribe_filtered(filter);

        // Emit event for other task (should be filtered out)
        let other_event = crate::observability::TaskEvent::new(
            other_task,
            UserId::new("alice"),
            TaskEventType::Started,
        );
        manager.emit(other_event).await.ok();

        // Emit event for target task (should be received)
        let target_event = crate::observability::TaskEvent::new(
            target_task,
            UserId::new("alice"),
            TaskEventType::Started,
        );
        manager.emit(target_event).await.ok();

        // Should only receive target task event
        let received = timeout(Duration::from_secs(1), receiver.recv())
            .await
            .expect("Timeout")
            .expect("Failed to receive");

        assert_eq!(received.task_id, target_task);
    }
}
