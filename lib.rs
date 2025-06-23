// Component modules
pub mod api;
pub mod communication;
pub mod discovery;
pub mod federation;
pub mod health;
pub mod network;
pub mod proxy;
pub mod registry;
pub mod security;

// Re-export security types
pub use security::{
    Action, AuthenticationProvider, AuthorizationProvider, Credentials, Permission, Resource,
    SecurityProvider, Subject,
};

// Re-export federation types
pub use federation::{
    FederatedServiceInfo, FederationConfig, FederationManager, FederationMessage,
    FederationMessageType, FederationMode, FederationRequest, FederationRequestType,
    FederationResponse, FederationStatus, McpFederation, ServiceProviderInfo,
};

// Re-export proxy types
pub use proxy::{
    CircuitBreakerState, CircuitState, ConnectionProxy, LoadBalancerState, LoadBalancingStrategy,
    ProxyConfig, ProxyRequest, ProxyResponse, ProxyStats,
}; 