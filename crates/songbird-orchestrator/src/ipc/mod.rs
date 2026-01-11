//! Inter-Primal Communication (IPC) via Unix Socket JSON-RPC
//!
//! v3.19.1: Modern async Rust patterns for primal-to-primal communication
//! v3.20.0: Service registry for capability-based primal discovery
//! v3.21.0: Graph intelligence APIs (Collaborative Intelligence)
//! v3.22.0: Pure Rust Unix socket implementation (BearDog pattern)
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────┐
//! │ Pure Rust Unix Socket Server (v3.22.0)               │
//! │ /run/user/{uid}/songbird-{family_id}.sock            │
//! ├──────────────────────────────────────────────────────┤
//! │ JSON-RPC 2.0 APIs (11 total):                        │
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
//! │                                                       │
//! │ Graph Intelligence (v3.21):                          │
//! │  • graph.validate (validate graph structure)         │
//! │  • graph.check_availability (check primal avail)     │
//! │  • graph.suggest_alternatives (suggest alts)         │
//! │  • coordination.validate_pattern (validate coord)    │
//! └──────────────────────────────────────────────────────┘
//! ```
//!
//! ## Design Principles
//!
//! 1. **Zero External RPC Libraries**: Pure tokio::net::UnixListener
//! 2. **Zero Hardcoding**: Socket path derived from env vars
//! 3. **Modern Async**: Pure tokio + async/await
//! 4. **Protocol Agnostic**: Standard JSON-RPC 2.0
//! 5. **Observable**: Structured logging at every step
//! 6. **Secure**: Unix socket permissions

pub mod handlers;
pub mod registry;
pub mod server_pure_rust; // v3.22.0: Pure Rust implementation

// Deprecated: Old jsonrpsee-based server
#[deprecated(note = "Use UnixSocketServer from server_pure_rust instead")]
pub mod server;

pub use registry::ServiceRegistry;
pub use server_pure_rust::UnixSocketServer; // v3.22.0: Pure Rust is default
pub use types::{
    // P2P Discovery (v3.19)
    AnnounceCapabilitiesRequest,
    AnnounceCapabilitiesResponse,
    CreateGeneticTunnelRequest,
    CreateGeneticTunnelResponse,
    DiscoverByCapabilityRequest,
    DiscoverByCapabilityResponse,
    DiscoverByFamilyRequest,
    DiscoverByFamilyResponse,
    GetServiceHealthRequest,
    GetServiceHealthResponse,
    HealthCheckRequest,
    HealthCheckResponse,
    HealthStatus,
    PrimalEndpoint,
    // Service Registry (v3.20)
    RegisterServiceRequest,
    RegisterServiceResponse,
};

pub mod types;
