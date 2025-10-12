//! Registry event types
//!
//! Events emitted by the registry for monitoring and reaction.

use crate::types::health::HealthStatus;
use crate::types::plugin::PluginId;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Types of events that can occur in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventType {
    /// Plugin was registered
    PluginRegistered {
        /// ID of the registered plugin
        plugin_id: PluginId,
    },

    /// Plugin was unregistered
    PluginUnregistered {
        /// ID of the unregistered plugin
        plugin_id: PluginId,
    },

    /// Plugin health changed
    HealthChanged {
        /// ID of the plugin
        plugin_id: PluginId,
        /// New health status
        status: HealthStatus,
    },

    /// Plugin was scaled
    PluginScaled {
        /// ID of the plugin
        plugin_id: PluginId,
        /// Previous instance count
        from: u32,
        /// New instance count
        to: u32,
    },

    /// Registry error occurred
    Error {
        /// Error message
        message: String,
        /// Optional plugin ID if error is plugin-specific
        plugin_id: Option<PluginId>,
    },
}

/// An event in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEvent {
    /// The type and details of the event
    #[serde(flatten)]
    pub event_type: EventType,

    /// When the event occurred
    pub timestamp: SystemTime,

    /// Optional additional context
    pub context: Option<serde_json::Value>,
}

impl RegistryEvent {
    /// Create a new event
    pub fn new(event_type: EventType) -> Self {
        Self {
            event_type,
            timestamp: SystemTime::now(),
            context: None,
        }
    }

    /// Add context to this event
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }

    /// Get the plugin ID associated with this event, if any
    pub fn plugin_id(&self) -> Option<&PluginId> {
        match &self.event_type {
            EventType::PluginRegistered {
                plugin_id,
            }
            | EventType::PluginUnregistered {
                plugin_id,
            }
            | EventType::HealthChanged {
                plugin_id,
                ..
            }
            | EventType::PluginScaled {
                plugin_id,
                ..
            } => Some(plugin_id),
            EventType::Error {
                plugin_id,
                ..
            } => plugin_id.as_ref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = RegistryEvent::new(EventType::PluginRegistered {
            plugin_id: PluginId::from("test"),
        });

        assert!(event.plugin_id().is_some());
        assert_eq!(event.plugin_id().unwrap().as_str(), "test");
    }

    #[test]
    fn test_event_with_context() {
        let event = RegistryEvent::new(EventType::Error {
            message: "Test error".to_string(),
            plugin_id: None,
        })
        .with_context(serde_json::json!({"severity": "high"}));

        assert!(event.context.is_some());
    }
}
