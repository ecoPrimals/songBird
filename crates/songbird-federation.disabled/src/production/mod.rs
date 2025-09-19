//! Production Federation Implementations Implementations
//!
//! This module contains production-ready federation implementations
//! that replace all mock and placeholder federation components.

pub mod real_federation_messaging;

pub use real_federation_messaging: :{ /// ProductionFederationMessaging, ProductionFederationMessaging,
    /// FederationMessagingConfig, FederationMessagingConfig,
    /// FederationNode, FederationNode,
    /// `NodeStatus`, NodeStatus,
    /// FederationMessage, FederationMessage,
    /// MessageType, MessageType,
    /// MessageAck, MessageAck,
    AckStatus};
