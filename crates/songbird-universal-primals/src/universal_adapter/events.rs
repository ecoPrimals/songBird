/// Universal Adapter Events
///
/// Event handling and broadcasting system for the universal adapter.
use super::types::*;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use tokio::sync::broadcast;
use uuid::Uuid;

/// Universal adapter events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UniversalAdapterEvent {
    /// Provider discovered
    ProviderDiscovered {
        provider: CapabilityProvider,
        discovered_at: SystemTime,
    },

    /// Provider became unavailable
    ProviderUnavailable {
        provider_id: Uuid,
        reason: String,
        timestamp: SystemTime,
    },

    /// Service registered
    ServiceRegistered {
        service: ServiceInstance,
        registered_at: SystemTime,
    },

    /// Service health changed
    ServiceHealthChanged {
        service_id: Uuid,
        old_health: ServiceHealthInfo,
        new_health: ServiceHealthInfo,
        timestamp: SystemTime,
    },

    /// Capability request processed
    CapabilityRequestProcessed {
        request_id: Uuid,
        capability_type: String,
        provider_id: Uuid,
        success: bool,
        response_time_ms: u64,
        timestamp: SystemTime,
    },

    /// System status changed
    SystemStatusChanged {
        old_status: String,
        new_status: String,
        timestamp: SystemTime,
    },

    /// Capability provider discovered (alias for ProviderDiscovered)
    CapabilityProviderDiscovered {
        provider_id: Uuid,
        capabilities: Vec<String>,
        endpoint: String,
        timestamp: SystemTime,
    },

    /// Capability not available for operation
    CapabilityNotAvailable {
        required_capability: String,
        operation: String,
        available_alternatives: Vec<String>,
        timestamp: SystemTime,
    },

    /// Operation routed to capability provider
    OperationRouted {
        operation: String,
        provider_id: Uuid,
        routing_decision: String,
        timestamp: SystemTime,
    },
}

/// Event broadcaster for the universal adapter
#[derive(Debug)]
pub struct EventBroadcaster {
    /// Event sender
    sender: broadcast::Sender<UniversalAdapterEvent>,

    /// Event receiver (for internal use)
    _receiver: broadcast::Receiver<UniversalAdapterEvent>,
}

impl EventBroadcaster {
    /// Create a new event broadcaster
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = broadcast::channel(capacity);
        Self {
            sender,
            _receiver: receiver,
        }
    }

    /// Broadcast an event
    pub fn broadcast(
        &self,
        event: UniversalAdapterEvent,
    ) -> Result<usize, broadcast::error::SendError<UniversalAdapterEvent>> {
        self.sender.send(event)
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<UniversalAdapterEvent> {
        self.sender.subscribe()
    }

    /// Get the number of active receivers
    pub fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBroadcaster {
    fn default() -> Self {
        Self::new(1000) // Default capacity of 1000 events
    }
}
