// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(missing_docs, reason = "variant names mirror `domain.verb` wire strings")]

/// `primal.*` methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimalMethod {
    /// `primal.info`
    Info,
    /// `primal.capabilities`
    Capabilities,
    /// `primal.register`
    Register,
    /// `primal.unregister`
    Unregister,
    /// `primal.get_provider`
    GetProvider,
    /// `primal.list_providers`
    ListProviders,
    /// `primal.list_all`
    ListAll,
    /// `primal.health`
    Health,
    /// `primal.ping`
    Ping,
}

/// `rpc.*` introspection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RpcMethod {
    /// `rpc.methods`
    Methods,
    /// `rpc.discover`
    Discover,
}

/// `health.*` (normalized ecosystem standard).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthMethod {
    /// `health.liveness`
    Liveness,
    /// `health.readiness`
    Readiness,
    /// `health.check`
    Check,
}

/// `capabilities.*`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CapabilitiesMethod {
    /// `capabilities.list`
    List,
    /// `capabilities.methods` — map of capability token → callable JSON-RPC methods
    Methods,
}

/// `ipc.*`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IpcMethod {
    Register,
    Resolve,
    Discover,
    List,
    FindCapability,
    Heartbeat,
}

/// `http.*`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    Request,
    Get,
    Post,
}

/// `stun.*`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StunMethod {
    Serve,
    Stop,
    Status,
    GetPublicAddress,
    Bind,
    ProbePortPattern,
    DetectNatType,
}

/// `igd.*`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IgdMethod {
    Discover,
    MapPort,
    UnmapPort,
    Status,
    ExternalIp,
    AutoConfigure,
}

/// `relay.*`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelayMethod {
    Serve,
    Stop,
    Status,
    Allocate,
}

/// `discovery.*` and legacy aliases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiscoveryMethod {
    Peers,
    Announce,
    ListPeers,
    PeerCount,
    RejectedPeers,
    Status,
}

/// `rendezvous.*`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RendezvousMethod {
    Register,
    Lookup,
}

/// `peer.*`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PeerMethod {
    Connect,
    Ping,
}

/// `birdsong.*`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BirdsongMethod {
    GenerateEncryptedBeacon,
    DecryptBeacon,
    VerifyLineage,
    GetLineage,
    Advertise,
    Schema,
}

/// `mesh.*`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeshMethod {
    Init,
    Status,
    FindPath,
    Announce,
    Peers,
    Topology,
    HealthCheck,
    AutoDiscover,
}

/// `punch.*`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PunchMethod {
    Request,
    Coordinate,
    Status,
}

/// `onion.*`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OnionMethod {
    Start,
    Stop,
    Status,
    Connect,
    Address,
}

/// `federation.*` / `songbird.federation.*`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FederationMethod {
    Peers,
    Status,
    Join,
}

/// `tor.*`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TorMethod {
    Status,
    Connect,
    ServiceStart,
    ServiceStop,
    ConsensusFetch,
    CircuitBuild,
    CircuitClose,
}

/// `compute.route`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComputeMethod {
    Route,
}

/// `songbird.compute.*`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SongbirdComputeMethod {
    Schedule,
    Status,
}

/// `deployment.*`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeploymentMethod {
    Create,
    Status,
}

/// `task.*`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskMethod {
    Create,
    List,
}

/// `consent.*`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConsentMethod {
    Check,
    Grant,
}

/// `registry.*`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegistryMethod {
    Register,
    Discover,
}

/// `protocol.negotiate` and `songbird.protocol.capabilities`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtocolMethod {
    Negotiate,
    Capabilities,
}

/// `songbird.services.*`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SongbirdServicesMethod {
    List,
    Get,
    Register,
}

/// `songbird.health` and `songbird.version`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SongbirdMethod {
    Health,
    Version,
}

/// `network.*`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetworkMethod {
    BeaconExchange,
    Broadcast,
    Listen,
}

/// `storage.*` — storage capability domain (delegated persistence; SB-03 migration surface).
///
/// When a storage capability provider exposes real `storage.*` IPC (NG-01), Songbird will call these
/// methods instead of embedding sled directly. Until then the enum is routed but
/// the sled backend remains the active implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StorageMethod {
    Get,
    Put,
    Delete,
    List,
    Flush,
}

/// Unix discovery encryption helpers (legacy names).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EncryptionDiscoveryMethod {
    Encrypt,
    Decrypt,
}
