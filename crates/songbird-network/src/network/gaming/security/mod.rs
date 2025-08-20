//! Security Provider Module
//!
//! Modular security provider system with universal primal integration

pub mod types;
pub mod providers;
pub mod tunnels;

// Re-export main types
pub use types::{
    SecurityCapabilityCache, SecurityPrimalInfo, SecurityProviderConfig,
    SecureTunnel, SecurityLevel, TunnelType, PeerInfo, TunnelStatus,
    SecurityStats, PrimalPerformanceMetrics,
};

// Re-export provider implementations
pub use providers::{
    UniversalSecurityProvider, NativeWireGuardProvider,
    UniversalSecurityManager, NoOpSecurityProvider,
}; 