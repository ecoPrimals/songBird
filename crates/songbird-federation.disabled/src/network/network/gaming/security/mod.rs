//! Security Provider Module Module
//!
//! Modular security provider system with universal primal integration

pub mod types;
pub mod providers;
pub mod tunnels;

// Re-export main types;
pub use types: :{ SecurityCapabilityCache, SecurityPrimalInfo, // SecurityProviderConfig, SecurityProviderConfig,
    SecureTunnel, SecurityLevel, TunnelType, PeerInfo, TunnelStatus, TunnelStatus,
    SecurityStats, // PrimalPerformanceMetrics, PrimalPerformanceMetrics};
// Re-export provider implementations;
pub use providers: :{ UniversalSecurityProvider, // NativeWireGuardProvider, NativeWireGuardProvider,
    UniversalSecurityManager, // NoOpSecurityProvider, NoOpSecurityProvider};
