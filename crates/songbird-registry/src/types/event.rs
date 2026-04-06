// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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
    #[must_use]
    pub fn new(event_type: EventType) -> Self {
        Self {
            event_type,
            timestamp: SystemTime::now(),
            context: None,
        }
    }

    /// Add context to this event
    #[must_use]
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }

    /// Get the plugin ID associated with this event, if any
    #[must_use]
    pub const fn plugin_id(&self) -> Option<&PluginId> {
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
        assert_eq!(event.plugin_id().expect("Test: plugin_id should be present").as_str(), "test");
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

    #[test]
    fn test_plugin_registered_event() {
        let plugin_id = PluginId::from("test-plugin");
        let event = RegistryEvent::new(EventType::PluginRegistered {
            plugin_id,
        });

        assert!(event.plugin_id().is_some(), "PluginRegistered event should have plugin_id");
        assert_eq!(event.plugin_id().unwrap().as_str(), "test-plugin", "Plugin ID should match");
    }

    #[test]
    fn test_plugin_unregistered_event() {
        let plugin_id = PluginId::from("unregister-test");
        let event = RegistryEvent::new(EventType::PluginUnregistered {
            plugin_id: plugin_id.clone(),
        });

        assert!(event.plugin_id().is_some(), "PluginUnregistered event should have plugin_id");
        assert_eq!(event.plugin_id().unwrap(), &plugin_id);
    }

    #[test]
    fn test_health_changed_event() {
        let plugin_id = PluginId::from("health-test");
        let event = RegistryEvent::new(EventType::HealthChanged {
            plugin_id: plugin_id.clone(),
            status: HealthStatus::healthy(),
        });

        assert!(event.plugin_id().is_some(), "HealthChanged event should have plugin_id");
        assert_eq!(event.plugin_id().unwrap(), &plugin_id);
    }

    #[test]
    fn test_plugin_scaled_event() {
        let plugin_id = PluginId::from("scale-test");
        let event = RegistryEvent::new(EventType::PluginScaled {
            plugin_id: plugin_id.clone(),
            from: 1,
            to: 3,
        });

        assert!(event.plugin_id().is_some(), "PluginScaled event should have plugin_id");
        assert_eq!(event.plugin_id().unwrap(), &plugin_id);
    }

    #[test]
    fn test_error_event_with_plugin_id() {
        let plugin_id = PluginId::from("error-test");
        let event = RegistryEvent::new(EventType::Error {
            message: "Test error".to_string(),
            plugin_id: Some(plugin_id.clone()),
        });

        assert!(event.plugin_id().is_some(), "Error event with plugin_id should return it");
        assert_eq!(event.plugin_id().unwrap(), &plugin_id);
    }

    #[test]
    fn test_error_event_without_plugin_id() {
        let event = RegistryEvent::new(EventType::Error {
            message: "General error".to_string(),
            plugin_id: None,
        });

        assert!(event.plugin_id().is_none(), "Error event without plugin_id should return None");
    }

    #[test]
    fn test_event_timestamp() {
        let before = SystemTime::now();
        let event = RegistryEvent::new(EventType::PluginRegistered {
            plugin_id: PluginId::from("timestamp-test"),
        });
        let after = SystemTime::now();

        assert!(
            event.timestamp >= before && event.timestamp <= after,
            "Event timestamp should be between before and after creation"
        );
    }

    #[test]
    fn test_event_context_chaining() {
        let event = RegistryEvent::new(EventType::PluginRegistered {
            plugin_id: PluginId::from("context-test"),
        })
        .with_context(serde_json::json!({
            "source": "test",
            "details": "additional info"
        }));

        assert!(event.context.is_some(), "Context should be set");
        let context = event.context.unwrap();
        assert!(context.get("source").is_some(), "Context should contain source field");
        assert!(context.get("details").is_some(), "Context should contain details field");
    }

    #[test]
    fn test_event_serialization() {
        let event = RegistryEvent::new(EventType::PluginRegistered {
            plugin_id: PluginId::from("serialize-test"),
        });

        let serialized = serde_json::to_string(&event);
        assert!(serialized.is_ok(), "Event should serialize to JSON successfully");

        let json_str = serialized.unwrap();
        assert!(json_str.contains("serialize-test"), "Serialized JSON should contain plugin ID");
    }

    #[test]
    fn test_event_deserialization() {
        let json = r#"{"type":"plugin_registered","plugin_id":"deserialize-test","timestamp":{"secs_since_epoch":0,"nanos_since_epoch":0},"context":null}"#;

        let result: Result<RegistryEvent, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "Event should deserialize from JSON successfully");

        let event = result.unwrap();
        assert!(event.plugin_id().is_some(), "Deserialized event should have plugin_id");
    }

    #[test]
    fn test_plugin_scaled_from_to_values() {
        let event = RegistryEvent::new(EventType::PluginScaled {
            plugin_id: PluginId::from("scale-values-test"),
            from: 2,
            to: 5,
        });

        if let EventType::PluginScaled {
            from,
            to,
            ..
        } = event.event_type
        {
            assert_eq!(from, 2, "From value should match");
            assert_eq!(to, 5, "To value should match");
            assert!(to > from, "Scale up: to should be greater than from");
        } else {
            panic!("Event type should be PluginScaled");
        }
    }

    #[test]
    fn test_error_event_message() {
        let error_message = "Critical failure detected";
        let event = RegistryEvent::new(EventType::Error {
            message: error_message.to_string(),
            plugin_id: None,
        });

        if let EventType::Error {
            message,
            ..
        } = event.event_type
        {
            assert_eq!(message, error_message, "Error message should match");
        } else {
            panic!("Event type should be Error");
        }
    }

    #[test]
    fn test_health_status_variants() {
        let statuses = vec![
            HealthStatus::healthy(),
            HealthStatus::degraded(0.5, "Degraded status"),
            HealthStatus::unhealthy("Unhealthy status"),
        ];

        for status in statuses {
            let event = RegistryEvent::new(EventType::HealthChanged {
                plugin_id: PluginId::from("health-variants-test"),
                status: status.clone(),
            });

            if let EventType::HealthChanged {
                status: event_status,
                ..
            } = event.event_type
            {
                // Health status should be preserved
                assert_eq!(
                    status.healthy, event_status.healthy,
                    "Health status healthy flag should match"
                );
                assert!(
                    (status.score - event_status.score).abs() < 0.01,
                    "Health status score should match"
                );
            } else {
                panic!("Event type should be HealthChanged");
            }
        }
    }
}
