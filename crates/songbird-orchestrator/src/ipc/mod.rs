//! Inter-Primal Communication (IPC) via Unix Socket JSON-RPC
//!
//! v3.19.1: Modern async Rust patterns for primal-to-primal communication
//! v3.20.0: Service registry for capability-based primal discovery
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────┐
//! │ Unix Socket Server                                   │
//! │ /run/user/{uid}/songbird-{family_id}.sock            │
//! ├──────────────────────────────────────────────────────┤
//! │ JSON-RPC 2.0 APIs:                                   │
//! │ P2P Discovery (v3.19):                               │
//! │  • discover_by_family (filter by genetic tags)       │
//! │  • create_genetic_tunnel (BTSP with genetic proof)   │
//! │  • announce_capabilities (update broadcaster)        │
//! │                                                       │
//! │ Service Registry (v3.20):                            │
//! │  • register_service (primals register themselves)    │
//! │  • discover_by_capability (find primals by cap)      │
//! │  • get_service_health (check primal health)          │
//! │  • health_check (Songbird's own health)              │
//! └──────────────────────────────────────────────────────┘
//! ```
//!
//! ## Design Principles
//!
//! 1. **Zero Hardcoding**: Socket path derived from env vars
//! 2. **Modern Async**: jsonrpsee with tokio
//! 3. **Protocol Agnostic**: Works with any JSON-RPC 2.0 client
//! 4. **Observable**: Structured logging at every step
//! 5. **Secure**: Unix socket permissions, credential passing (future)

pub mod handlers;
pub mod server;
pub mod types;
pub mod registry;

pub use server::UnixSocketServer;
pub use registry::ServiceRegistry;
pub use types::{
    // P2P Discovery (v3.19)
    AnnounceCapabilitiesRequest, AnnounceCapabilitiesResponse, CreateGeneticTunnelRequest,
    CreateGeneticTunnelResponse, DiscoverByFamilyRequest, DiscoverByFamilyResponse,
    // Service Registry (v3.20)
    RegisterServiceRequest, RegisterServiceResponse, DiscoverByCapabilityRequest,
    DiscoverByCapabilityResponse, GetServiceHealthRequest, GetServiceHealthResponse,
    HealthCheckRequest, HealthCheckResponse, PrimalEndpoint, HealthStatus,
};
