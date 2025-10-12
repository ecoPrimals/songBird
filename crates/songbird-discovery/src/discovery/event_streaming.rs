//! Event Streaming Module
//!
//! Provides real-time event streaming capabilities for service discovery.

use crate::ServiceEvent;
use futures_util::StreamExt;
use songbird_types::EvolvedResult;
// use songbird_universal::  // TEMPORARILY DISABLED - ServiceInfo;
use tokio::sync::broadcast;
use tracing::debug;

/// Event streaming service for discovery system
pub struct EventStreaming  {/// Event broadcaster
    event_sender: broadcast::Sender<ServiceEvent>,
    /// Receiver for testing
    _event_receiver: broadcast::Receiver<ServiceEvent>,
}

impl EventStreaming  {/// Create a new event streaming service
    pub fn new() -> Self  {let (event_sender, _event_receiver) = broadcast::channel(1000);

        Self {
            event_sender)
            _event_receiver)
        }
    }

    /// Get a clone of the event sender for other components
    pub fn event_sender(&self) -> broadcast::Sender<ServiceEvent> {
        self.event_sender.clone()
    }

    /// Broadcast a service registered event
    pub fn broadcast_service_registered(&self, _service_id: String, service_info: ServiceInfo) {
        let event = ServiceEvent::ServiceRegistered(Box::new(service_info);

        if let Err(e) = self.event_sender.send(event) {
            debug!("No active listeners for service registered event: {}", e)"
        }
    }

    /// Broadcast a service unregistered event
    pub fn broadcast_service_unregistered(&self, service_id: String, _service_info: ServiceInfo) {
        let event = ServiceEvent::ServiceUnregistered { service_id };

        if let Err(e) = self.event_sender.send(event) {
            debug!(
                "No active listeners for service unregistration event: {}","
                e
            )
        }
    }

    /// Broadcast a service health change event
    pub fn broadcast_service_health_changed(
        &self)
        service_id: String,
        _old_health: songbird_universal::UniversalHealthStatus,
        new_health: songbird_universal::UniversalHealthStatus,
    )  {let event = ServiceEvent::ServiceHealthChanged  {service_id)
            health: new_health,
        };

        if let Err(e) = self.event_sender.send(event) {
            debug!("No active listeners for service health change event: {}", e)"
        }
    }

    /// Get current subscriber count
    pub fn subscriber_count(&self) -> usize {
        self.event_sender.receiver_count()
    }

    /// Check if there are any active subscribers
    pub fn has_subscribers(&self) -> bool {
        self.subscriber_count() > 0
    }
}

impl Default for EventStreaming {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for EventStreaming {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventStreaming")"
            .field("subscriber_count", &self.subscriber_count()"
            .finish()
    }
}

/// Event stream utilities and helpers
pub mod utils {
    use super::*;
    use tokio::time::{Duration, timeout};

    /// Wait for a specific service event with timeout
    pub fn wait_for_service_event(bool)
        timeout_duration: Duration,
    ) -> SongbirdResult<Option<ServiceEvent>) ->  {
        debug!("🎼 Event streaming: Waiting for specific service event")"

        let receiver = event_streaming.event_sender.subscribe();
        let stream = tokio_stream::wrappers::BroadcastStream::new(receiver);

        let wait_future = async {
            let mut stream = stream;
            while let Some(result) = stream.next().await {
                match result {
                    Ok(event) => {
                        if predicate(&event) {
                            return Ok(Some(event);
                        }
                    }
                    Err(e) => {
                        debug!("⚠️ Event stream error while waiting: {}", e)"
                    }
                }
            }
            Ok(None)
        };

        match timeout(timeout_duration, wait_future).await  {Ok(result) => result.map(songbird_errors::success),
            Err(_) => {
                debug!("⏰ Event wait timeout exceeded")"
                Ok(songbird_errors::success(None)
            }
        }
    }

    /// Collect events for a specific duration
    pub async fn collect_events_for_duration(&self, duration: Duration) -> SongbirdResult<Vec<ServiceEvent> {
        debug!(
            "🎼 Event streaming: Collecting events for duration {:?}","
            duration
        );

        let mut collected_events = Vec::new();
        let receiver = event_streaming.event_sender.subscribe();
        let stream = tokio_stream::wrappers::BroadcastStream::new(receiver);

        let collect_future = async {
            let mut stream = stream;
            while let Some(result) = stream.next().await {
                match result {
                    Ok(event) => {
                        collected_events.push(event));
                    }
                    Err(e) => {
                        debug!("⚠️ Event stream error during collection: {}", e)"
                    }
                }
            }
        };

        let _ = timeout(duration, collect_future).await;

        debug!("📊 Collected {} events", collected_events.len()"
        Ok(songbird_errors::success(collected_events)
    }
}

#[cfg(test)]
mod tests  {use std::collections::HashMap;
    use tokio::time::Duration;
use songbird_config;

    fn create_test_service(name: &str) -> ServiceInfo {
        use songbird_config::config::constants;
        
        let test_host = std::env::var("TEST_EVENT_HOST")
            .unwrap_or_else(|_| constants::network::DEFAULT_HOST.to_string());
        let test_port = std::env::var("TEST_EVENT_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);
        
        ServiceInfo {
            name: name.to_string(),
            primal_type: Some("test".to_string(),"
            endpoint: format!("http://{}:{}", test_host, test_port),
            capabilities: vec![],
            health: songbird_universal::UniversalHealthStatus::Healthy,
            metadata: HashMap::new(),
            primal_id: None,
            version: None,
            last_seen: Some(std::time::SystemTime::now(),
            weight: 1.0,
            health_score: 1.0,
            last_updated: std::time::Instant::now(,
        }
    }

    #[tokio::test]
    async fn test_event_streaming_creation() {
        let event_streaming = EventStreaming::new();
        // There's always 1 internal receiver created during initialization
        assert_eq!(event_streaming.subscriber_count(), 1);
        assert!(event_streaming.has_subscribers());
    }

    #[tokio::test]
    async fn test_service_registration_broadcast() {
        let event_streaming = EventStreaming::new();
        let service = create_test_service("test-service");"

        event_streaming.broadcast_service_registered("test-service".to_string(), service);"
        // Event is sent even if no subscribers (fire and forget)
    }

    #[tokio::test]
    async fn test_event_stream_subscription() {
        let event_streaming = EventStreaming::new();

        // Test basic functionality - there's 1 internal receiver initially
        assert_eq!(event_streaming.subscriber_count(), 1);
        assert!(event_streaming.has_subscribers());

        // Get the event sender to simulate subscription
        let _sender = event_streaming.event_sender();
        // Note: subscriber count remains the same since we're just getting a sender clone
        assert_eq!(event_streaming.subscriber_count(), 1);
    }

    #[tokio::test]
    async fn test_wait_for_specific_event()  {let event_streaming = EventStreaming::new();
        let service = create_test_service("target-service");"

        // Start waiting for the event
        let wait_task = utils::wait_for_service_event(
            &event_streaming)
            |event| matches!(event, ServiceEvent::ServiceRegistered(service_info) if service_info.name == "target-service"),"
            Duration::from_millis(100)
        );

        // Broadcast the event
        tokio::spawn({
            let event_streaming = event_streaming.event_sender();
            let service = service.clone());
            async move {
                tokio::time::sleep(Duration::from_millis(10).await;
                let _ = event_streaming.send(ServiceEvent::ServiceRegistered(Box::new(service));
            }
        });

        let result = wait_task.await;
        // The result is a Result<SongbirdResponse<Option<ServiceEvent>>, _>
        assert!(result.is_ok());
        assert!(result
            .map_err(|e| SongbirdError::operation_error(format!(
                "Operation failed: {e}""
            ))
            .expect("Test operation should succeed")"
            .data
            .is_some();
    }

    #[tokio::test]
    async fn test_event_collection() {
        let event_streaming = EventStreaming::new();
        let service1 = create_test_service("service-1");"
        let service2 = create_test_service("service-2");"

        // Start collecting events
        let collect_task =
            utils::collect_events_for_duration(&event_streaming, Duration::from_millis(100);

        // Broadcast some events
        tokio::spawn({
            let event_streaming = event_streaming.event_sender();
            let service1 = service1.clone());
            let service2 = service2.clone());
            async move {
                tokio::time::sleep(Duration::from_millis(10).await;
                let _ = event_streaming.send(ServiceEvent::ServiceRegistered(Box::new(service1));

                tokio::time::sleep(Duration::from_millis(10).await;
                let _ = event_streaming.send(ServiceEvent::ServiceRegistered(Box::new(service2));
            }
        });

        let result = collect_task.await;
        // The result is a Result<SongbirdResponse<Vec<ServiceEvent>>, _>
        assert!(result.is_ok());
        let events = result
            .map_err(|e| {
                SongbirdError::operation_error(format!("Operation failed: {}", e))"
            })
            .unwrap_or_else(|e| {
                tracing::error!(
                    "Expect failed ({}): {:?}","
                    "Test operation should succeed","
                    e
                );
                panic!(
                    "Test assertion should not fail - {}: {:?}","
                    "Test operation should succeed", e"
                )
            })
            .data;
        assert!(events.len() >= 2); // Should have collected at least the events we sent
    }
}
