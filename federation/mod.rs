/*!
 * Federation management for Songbird Orchestrator
 * 
 * This module provides distributed service federation capabilities including:
 * - MCP (Model Context Protocol) federation for distributed orchestration
 * - Multi-node cluster management and coordination
 * - Service discovery and registration across federation nodes
 * - Heartbeat and health monitoring for federated nodes
 * - Storage provider registration and management
 * - Cross-cluster communication and request handling
 * 
 * ## Architecture
 * 
 * The federation system is organized into focused modules:
 * - `config`: Federation configuration types and enums
 * - `messages`: Message types and data structures for communication
 * - `mcp_handler`: Core MCP federation implementation
 * - `manager`: High-level federation coordination and management
 */

// Module declarations
pub mod config;
pub mod messages;
pub mod mcp_handler;
pub mod manager;

// Re-export main types for easy access
pub use config::{FederationMode, FederationConfig, FederationStatus};
pub use messages::{
    ServiceProviderInfo, FederationRequest, FederationResponse, FederationRequestType,
    FederatedServiceInfo, FederationMessage, FederationMessageType
};
pub use mcp_handler::McpFederation;
pub use manager::FederationManager;

// Convenience type aliases
pub type FederationResult<T> = Result<T, crate::errors::SongbirdError>;

/// Default federation configuration for quick setup
pub fn default_federation_config() -> FederationConfig {
    FederationConfig::default()
}

/// Create a new federation manager with default settings
pub fn create_federation_manager(mode: FederationMode) -> FederationManager {
    FederationManager::new(mode)
}

/// Check if a federation mode requires network connectivity
pub fn mode_requires_network(mode: &FederationMode) -> bool {
    !matches!(mode, FederationMode::Standalone)
} 