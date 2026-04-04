// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(missing_docs, reason = "variant names mirror `domain.verb` wire strings")]
#![allow(
    clippy::too_many_lines,
    reason = "`as_wire_str` / `from_wire_str` are exhaustive mechanical dispatch tables"
)]

//! Typed JSON-RPC 2.0 method names (`domain.verb`) for Songbird dispatch.
//!
//! Wire format stays plain strings; this module maps those strings to enums for
//! exhaustive routing while [`std::fmt::Display`] / serde serialize back to canonical names.

use core::fmt;
use core::str::FromStr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

mod domain_methods;

pub use domain_methods::*;

/// Normalize known aliases before parsing (same rules as `songbird-universal-ipc` introspection).
#[must_use]
pub fn normalize_json_rpc_method_name(method: &str) -> &str {
    match method {
        "capability.list" => "capabilities.list",
        "ping" => "health.liveness",
        "register_service" => "ipc.register",
        "health_check" | "status" | "check" | "health" => "health.check",
        other => other,
    }
}

/// Full method table for Songbird JSON-RPC dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JsonRpcMethod {
    Primal(PrimalMethod),
    Rpc(RpcMethod),
    DiscoverCapabilities,
    Identity,
    /// Raw `health` on the orchestrator Unix socket (biomeOS); not `health.check`.
    BiomeOsHealth,
    Health(HealthMethod),
    Capabilities(CapabilitiesMethod),
    Ipc(IpcMethod),
    Http(HttpMethod),
    Stun(StunMethod),
    Igd(IgdMethod),
    Relay(RelayMethod),
    Discovery(DiscoveryMethod),
    Rendezvous(RendezvousMethod),
    Peer(PeerMethod),
    Birdsong(BirdsongMethod),
    Mesh(MeshMethod),
    Punch(PunchMethod),
    Onion(OnionMethod),
    Federation(FederationMethod),
    Tor(TorMethod),
    Compute(ComputeMethod),
    SongbirdCompute(SongbirdComputeMethod),
    Deployment(DeploymentMethod),
    Task(TaskMethod),
    Consent(ConsentMethod),
    Registry(RegistryMethod),
    Protocol(ProtocolMethod),
    SongbirdServices(SongbirdServicesMethod),
    Songbird(SongbirdMethod),
    Network(NetworkMethod),
    Storage(StorageMethod),
    EncryptionDiscovery(EncryptionDiscoveryMethod),
}

/// Failed to map a wire string to [`JsonRpcMethod`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unknown JSON-RPC method: {0}")]
pub struct JsonRpcMethodParseError(pub String);

impl JsonRpcMethod {
    /// Primary wire name for this method (used by [`std::fmt::Display`] and JSON serialization).
    #[must_use]
    pub const fn as_wire_str(self) -> &'static str {
        match self {
            Self::Primal(PrimalMethod::Info) => "primal.info",
            Self::Primal(PrimalMethod::Capabilities) => "primal.capabilities",
            Self::Primal(PrimalMethod::Register) => "primal.register",
            Self::Primal(PrimalMethod::Unregister) => "primal.unregister",
            Self::Primal(PrimalMethod::GetProvider) => "primal.get_provider",
            Self::Primal(PrimalMethod::ListProviders) => "primal.list_providers",
            Self::Primal(PrimalMethod::ListAll) => "primal.list_all",
            Self::Primal(PrimalMethod::Health) => "primal.health",
            Self::Primal(PrimalMethod::Ping) => "primal.ping",
            Self::Rpc(RpcMethod::Methods) => "rpc.methods",
            Self::Rpc(RpcMethod::Discover) => "rpc.discover",
            Self::DiscoverCapabilities => "discover_capabilities",
            Self::Identity => "identity",
            Self::BiomeOsHealth => "health",
            Self::Health(HealthMethod::Liveness) => "health.liveness",
            Self::Health(HealthMethod::Readiness) => "health.readiness",
            Self::Health(HealthMethod::Check) => "health.check",
            Self::Capabilities(CapabilitiesMethod::List) => "capabilities.list",
            Self::Ipc(IpcMethod::Register) => "ipc.register",
            Self::Ipc(IpcMethod::Resolve) => "ipc.resolve",
            Self::Ipc(IpcMethod::Discover) => "ipc.discover",
            Self::Ipc(IpcMethod::List) => "ipc.list",
            Self::Ipc(IpcMethod::FindCapability) => "ipc.find_capability",
            Self::Ipc(IpcMethod::Heartbeat) => "ipc.heartbeat",
            Self::Http(HttpMethod::Request) => "http.request",
            Self::Http(HttpMethod::Get) => "http.get",
            Self::Http(HttpMethod::Post) => "http.post",
            Self::Stun(StunMethod::Serve) => "stun.serve",
            Self::Stun(StunMethod::Stop) => "stun.stop",
            Self::Stun(StunMethod::Status) => "stun.status",
            Self::Stun(StunMethod::GetPublicAddress) => "stun.get_public_address",
            Self::Stun(StunMethod::Bind) => "stun.bind",
            Self::Stun(StunMethod::ProbePortPattern) => "stun.probe_port_pattern",
            Self::Stun(StunMethod::DetectNatType) => "stun.detect_nat_type",
            Self::Igd(IgdMethod::Discover) => "igd.discover",
            Self::Igd(IgdMethod::MapPort) => "igd.map_port",
            Self::Igd(IgdMethod::UnmapPort) => "igd.unmap_port",
            Self::Igd(IgdMethod::Status) => "igd.status",
            Self::Igd(IgdMethod::ExternalIp) => "igd.external_ip",
            Self::Igd(IgdMethod::AutoConfigure) => "igd.auto_configure",
            Self::Relay(RelayMethod::Serve) => "relay.serve",
            Self::Relay(RelayMethod::Stop) => "relay.stop",
            Self::Relay(RelayMethod::Status) => "relay.status",
            Self::Relay(RelayMethod::Allocate) => "relay.allocate",
            Self::Discovery(DiscoveryMethod::Peers) => "discovery.peers",
            Self::Discovery(DiscoveryMethod::Announce) => "discovery.announce",
            Self::Discovery(DiscoveryMethod::ListPeers) => "discovery.list_peers",
            Self::Discovery(DiscoveryMethod::PeerCount) => "discovery.peer_count",
            Self::Discovery(DiscoveryMethod::RejectedPeers) => "discovery.rejected_peers",
            Self::Discovery(DiscoveryMethod::Status) => "discovery.status",
            Self::Rendezvous(RendezvousMethod::Register) => "rendezvous.register",
            Self::Rendezvous(RendezvousMethod::Lookup) => "rendezvous.lookup",
            Self::Peer(PeerMethod::Connect) => "peer.connect",
            Self::Peer(PeerMethod::Ping) => "peer.ping",
            Self::Birdsong(BirdsongMethod::GenerateEncryptedBeacon) => {
                "birdsong.generate_encrypted_beacon"
            }
            Self::Birdsong(BirdsongMethod::DecryptBeacon) => "birdsong.decrypt_beacon",
            Self::Birdsong(BirdsongMethod::VerifyLineage) => "birdsong.verify_lineage",
            Self::Birdsong(BirdsongMethod::GetLineage) => "birdsong.get_lineage",
            Self::Birdsong(BirdsongMethod::Advertise) => "birdsong.advertise",
            Self::Birdsong(BirdsongMethod::Schema) => "birdsong.schema",
            Self::Mesh(MeshMethod::Init) => "mesh.init",
            Self::Mesh(MeshMethod::Status) => "mesh.status",
            Self::Mesh(MeshMethod::FindPath) => "mesh.find_path",
            Self::Mesh(MeshMethod::Announce) => "mesh.announce",
            Self::Mesh(MeshMethod::Peers) => "mesh.peers",
            Self::Mesh(MeshMethod::Topology) => "mesh.topology",
            Self::Mesh(MeshMethod::HealthCheck) => "mesh.health_check",
            Self::Mesh(MeshMethod::AutoDiscover) => "mesh.auto_discover",
            Self::Punch(PunchMethod::Request) => "punch.request",
            Self::Punch(PunchMethod::Coordinate) => "punch.coordinate",
            Self::Punch(PunchMethod::Status) => "punch.status",
            Self::Onion(OnionMethod::Start) => "onion.start",
            Self::Onion(OnionMethod::Stop) => "onion.stop",
            Self::Onion(OnionMethod::Status) => "onion.status",
            Self::Onion(OnionMethod::Connect) => "onion.connect",
            Self::Onion(OnionMethod::Address) => "onion.address",
            Self::Federation(FederationMethod::Peers) => "songbird.federation.peers",
            Self::Federation(FederationMethod::Status) => "songbird.federation.status",
            Self::Federation(FederationMethod::Join) => "songbird.federation.join",
            Self::Tor(TorMethod::Status) => "tor.status",
            Self::Tor(TorMethod::Connect) => "tor.connect",
            Self::Tor(TorMethod::ServiceStart) => "tor.service.start",
            Self::Tor(TorMethod::ServiceStop) => "tor.service.stop",
            Self::Tor(TorMethod::ConsensusFetch) => "tor.consensus.fetch",
            Self::Tor(TorMethod::CircuitBuild) => "tor.circuit.build",
            Self::Tor(TorMethod::CircuitClose) => "tor.circuit.close",
            Self::Compute(ComputeMethod::Route) => "compute.route",
            Self::SongbirdCompute(SongbirdComputeMethod::Schedule) => "songbird.compute.schedule",
            Self::SongbirdCompute(SongbirdComputeMethod::Status) => "songbird.compute.status",
            Self::Deployment(DeploymentMethod::Create) => "deployment.create",
            Self::Deployment(DeploymentMethod::Status) => "deployment.status",
            Self::Task(TaskMethod::Create) => "task.create",
            Self::Task(TaskMethod::List) => "task.list",
            Self::Consent(ConsentMethod::Check) => "consent.check",
            Self::Consent(ConsentMethod::Grant) => "consent.grant",
            Self::Registry(RegistryMethod::Register) => "registry.register",
            Self::Registry(RegistryMethod::Discover) => "registry.discover",
            Self::Protocol(ProtocolMethod::Negotiate) => "protocol.negotiate",
            Self::Protocol(ProtocolMethod::Capabilities) => "songbird.protocol.capabilities",
            Self::SongbirdServices(SongbirdServicesMethod::List) => "songbird.services.list",
            Self::SongbirdServices(SongbirdServicesMethod::Get) => "songbird.services.get",
            Self::SongbirdServices(SongbirdServicesMethod::Register) => {
                "songbird.services.register"
            }
            Self::Songbird(SongbirdMethod::Health) => "songbird.health",
            Self::Songbird(SongbirdMethod::Version) => "songbird.version",
            Self::Network(NetworkMethod::BeaconExchange) => "network.beacon_exchange",
            Self::Network(NetworkMethod::Broadcast) => "network.broadcast",
            Self::Network(NetworkMethod::Listen) => "network.listen",
            Self::Storage(StorageMethod::Get) => "storage.get",
            Self::Storage(StorageMethod::Put) => "storage.put",
            Self::Storage(StorageMethod::Delete) => "storage.delete",
            Self::Storage(StorageMethod::List) => "storage.list",
            Self::Storage(StorageMethod::Flush) => "storage.flush",
            Self::EncryptionDiscovery(EncryptionDiscoveryMethod::Encrypt) => "encrypt_discovery",
            Self::EncryptionDiscovery(EncryptionDiscoveryMethod::Decrypt) => "decrypt_discovery",
        }
    }

    /// Parse a raw JSON-RPC `method` string (as sent on the wire before alias normalization).
    ///
    /// Use this for the orchestrator Unix socket, which routes bare `"health"` separately from
    /// `"health.check"`.
    ///
    /// # Errors
    ///
    /// Returns [`JsonRpcMethodParseError`] when the name is not recognized.
    pub fn from_wire_str(s: &str) -> Result<Self, JsonRpcMethodParseError> {
        Ok(match s {
            "primal.info" => Self::Primal(PrimalMethod::Info),
            "primal.capabilities" => Self::Primal(PrimalMethod::Capabilities),
            "primal.register" => Self::Primal(PrimalMethod::Register),
            "primal.unregister" => Self::Primal(PrimalMethod::Unregister),
            "primal.get_provider" => Self::Primal(PrimalMethod::GetProvider),
            "primal.list_providers" => Self::Primal(PrimalMethod::ListProviders),
            "primal.list_all" => Self::Primal(PrimalMethod::ListAll),
            "primal.health" => Self::Primal(PrimalMethod::Health),
            "primal.ping" => Self::Primal(PrimalMethod::Ping),
            "rpc.methods" => Self::Rpc(RpcMethod::Methods),
            "rpc.discover" => Self::Rpc(RpcMethod::Discover),
            "discover_capabilities" => Self::DiscoverCapabilities,
            "identity" => Self::Identity,
            "health" => Self::BiomeOsHealth,
            "health.liveness" => Self::Health(HealthMethod::Liveness),
            "health.readiness" => Self::Health(HealthMethod::Readiness),
            "health.check" => Self::Health(HealthMethod::Check),
            "capabilities.list" => Self::Capabilities(CapabilitiesMethod::List),
            "ipc.register" => Self::Ipc(IpcMethod::Register),
            "ipc.resolve" => Self::Ipc(IpcMethod::Resolve),
            "ipc.discover" => Self::Ipc(IpcMethod::Discover),
            "ipc.list" => Self::Ipc(IpcMethod::List),
            "ipc.find_capability" => Self::Ipc(IpcMethod::FindCapability),
            "ipc.heartbeat" => Self::Ipc(IpcMethod::Heartbeat),
            "http.request" => Self::Http(HttpMethod::Request),
            "http.get" => Self::Http(HttpMethod::Get),
            "http.post" => Self::Http(HttpMethod::Post),
            "stun.serve" => Self::Stun(StunMethod::Serve),
            "stun.stop" => Self::Stun(StunMethod::Stop),
            "stun.status" => Self::Stun(StunMethod::Status),
            "stun.get_public_address" => Self::Stun(StunMethod::GetPublicAddress),
            "stun.bind" => Self::Stun(StunMethod::Bind),
            "stun.probe_port_pattern" => Self::Stun(StunMethod::ProbePortPattern),
            "stun.detect_nat_type" => Self::Stun(StunMethod::DetectNatType),
            "igd.discover" => Self::Igd(IgdMethod::Discover),
            "igd.map_port" => Self::Igd(IgdMethod::MapPort),
            "igd.unmap_port" => Self::Igd(IgdMethod::UnmapPort),
            "igd.status" => Self::Igd(IgdMethod::Status),
            "igd.external_ip" => Self::Igd(IgdMethod::ExternalIp),
            "igd.auto_configure" => Self::Igd(IgdMethod::AutoConfigure),
            "relay.serve" => Self::Relay(RelayMethod::Serve),
            "relay.stop" => Self::Relay(RelayMethod::Stop),
            "relay.status" => Self::Relay(RelayMethod::Status),
            "relay.allocate" => Self::Relay(RelayMethod::Allocate),
            "discovery.peers" | "discovery.find_primals" | "find_primals" => {
                Self::Discovery(DiscoveryMethod::Peers)
            }
            "discovery.announce" | "announce_presence" => {
                Self::Discovery(DiscoveryMethod::Announce)
            }
            "discovery.list_peers" => Self::Discovery(DiscoveryMethod::ListPeers),
            "discovery.peer_count" => Self::Discovery(DiscoveryMethod::PeerCount),
            "discovery.rejected_peers" => Self::Discovery(DiscoveryMethod::RejectedPeers),
            "discovery.status" => Self::Discovery(DiscoveryMethod::Status),
            "rendezvous.register" => Self::Rendezvous(RendezvousMethod::Register),
            "rendezvous.lookup" => Self::Rendezvous(RendezvousMethod::Lookup),
            "peer.connect" => Self::Peer(PeerMethod::Connect),
            "peer.ping" => Self::Peer(PeerMethod::Ping),
            "birdsong.generate_encrypted_beacon" => {
                Self::Birdsong(BirdsongMethod::GenerateEncryptedBeacon)
            }
            "birdsong.decrypt_beacon" => Self::Birdsong(BirdsongMethod::DecryptBeacon),
            "birdsong.verify_lineage" => Self::Birdsong(BirdsongMethod::VerifyLineage),
            "birdsong.get_lineage" => Self::Birdsong(BirdsongMethod::GetLineage),
            "birdsong.advertise" => Self::Birdsong(BirdsongMethod::Advertise),
            "birdsong.schema" => Self::Birdsong(BirdsongMethod::Schema),
            "mesh.init" => Self::Mesh(MeshMethod::Init),
            "mesh.status" => Self::Mesh(MeshMethod::Status),
            "mesh.find_path" => Self::Mesh(MeshMethod::FindPath),
            "mesh.announce" => Self::Mesh(MeshMethod::Announce),
            "mesh.peers" => Self::Mesh(MeshMethod::Peers),
            "mesh.topology" => Self::Mesh(MeshMethod::Topology),
            "mesh.health_check" => Self::Mesh(MeshMethod::HealthCheck),
            "mesh.auto_discover" => Self::Mesh(MeshMethod::AutoDiscover),
            "punch.request" => Self::Punch(PunchMethod::Request),
            "punch.coordinate" => Self::Punch(PunchMethod::Coordinate),
            "punch.status" => Self::Punch(PunchMethod::Status),
            "onion.start" => Self::Onion(OnionMethod::Start),
            "onion.stop" => Self::Onion(OnionMethod::Stop),
            "onion.status" => Self::Onion(OnionMethod::Status),
            "onion.connect" => Self::Onion(OnionMethod::Connect),
            "onion.address" => Self::Onion(OnionMethod::Address),
            "songbird.federation.peers" | "federation.peers" => {
                Self::Federation(FederationMethod::Peers)
            }
            "songbird.federation.status" | "federation.status" => {
                Self::Federation(FederationMethod::Status)
            }
            "songbird.federation.join" => Self::Federation(FederationMethod::Join),
            "tor.status" => Self::Tor(TorMethod::Status),
            "tor.connect" => Self::Tor(TorMethod::Connect),
            "tor.service.start" => Self::Tor(TorMethod::ServiceStart),
            "tor.service.stop" => Self::Tor(TorMethod::ServiceStop),
            "tor.consensus.fetch" => Self::Tor(TorMethod::ConsensusFetch),
            "tor.circuit.build" => Self::Tor(TorMethod::CircuitBuild),
            "tor.circuit.close" => Self::Tor(TorMethod::CircuitClose),
            "compute.route" => Self::Compute(ComputeMethod::Route),
            "songbird.compute.schedule" => Self::SongbirdCompute(SongbirdComputeMethod::Schedule),
            "songbird.compute.status" => Self::SongbirdCompute(SongbirdComputeMethod::Status),
            "deployment.create" => Self::Deployment(DeploymentMethod::Create),
            "deployment.status" => Self::Deployment(DeploymentMethod::Status),
            "task.create" => Self::Task(TaskMethod::Create),
            "task.list" => Self::Task(TaskMethod::List),
            "consent.check" => Self::Consent(ConsentMethod::Check),
            "consent.grant" => Self::Consent(ConsentMethod::Grant),
            "registry.register" => Self::Registry(RegistryMethod::Register),
            "registry.discover" => Self::Registry(RegistryMethod::Discover),
            "protocol.negotiate" => Self::Protocol(ProtocolMethod::Negotiate),
            "songbird.protocol.capabilities" => Self::Protocol(ProtocolMethod::Capabilities),
            "songbird.services.list" => Self::SongbirdServices(SongbirdServicesMethod::List),
            "songbird.services.get" => Self::SongbirdServices(SongbirdServicesMethod::Get),
            "songbird.services.register" => {
                Self::SongbirdServices(SongbirdServicesMethod::Register)
            }
            "songbird.health" => Self::Songbird(SongbirdMethod::Health),
            "songbird.version" => Self::Songbird(SongbirdMethod::Version),
            "network.beacon_exchange" => Self::Network(NetworkMethod::BeaconExchange),
            "network.broadcast" => Self::Network(NetworkMethod::Broadcast),
            "network.listen" => Self::Network(NetworkMethod::Listen),
            "storage.get" => Self::Storage(StorageMethod::Get),
            "storage.put" => Self::Storage(StorageMethod::Put),
            "storage.delete" => Self::Storage(StorageMethod::Delete),
            "storage.list" => Self::Storage(StorageMethod::List),
            "storage.flush" => Self::Storage(StorageMethod::Flush),
            "encrypt_discovery" => Self::EncryptionDiscovery(EncryptionDiscoveryMethod::Encrypt),
            "decrypt_discovery" => Self::EncryptionDiscovery(EncryptionDiscoveryMethod::Decrypt),
            _ => return Err(JsonRpcMethodParseError(s.to_string())),
        })
    }

    /// Parse after applying [`normalize_json_rpc_method_name`] (IPC broker and HTTP gateway).
    ///
    /// # Errors
    ///
    /// Returns [`JsonRpcMethodParseError`] when the name is not recognized.
    pub fn parse_ipc(method: &str) -> Result<Self, JsonRpcMethodParseError> {
        Self::from_wire_str(normalize_json_rpc_method_name(method))
    }
}

impl fmt::Display for JsonRpcMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_wire_str())
    }
}

impl FromStr for JsonRpcMethod {
    type Err = JsonRpcMethodParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_wire_str(s)
    }
}

impl Serialize for JsonRpcMethod {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_wire_str())
    }
}

impl<'de> Deserialize<'de> for JsonRpcMethod {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod json_rpc_method_tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn parse_ipc_normalizes_aliases() {
        assert_eq!(
            JsonRpcMethod::parse_ipc("ping").unwrap(),
            JsonRpcMethod::Health(HealthMethod::Liveness)
        );
        assert_eq!(
            JsonRpcMethod::parse_ipc("health").unwrap(),
            JsonRpcMethod::Health(HealthMethod::Check)
        );
    }

    #[test]
    fn roundtrip_display_from_str() {
        let m = JsonRpcMethod::Discovery(DiscoveryMethod::Peers);
        assert_eq!(m.to_string(), "discovery.peers");
        assert_eq!(JsonRpcMethod::from_str("discovery.peers").unwrap(), m);
        assert_eq!(JsonRpcMethod::from_str("find_primals").unwrap(), m);
    }

    #[test]
    fn serde_json_roundtrip() {
        let m = JsonRpcMethod::Federation(FederationMethod::Peers);
        let v = serde_json::to_string(&m).unwrap();
        assert_eq!(v, "\"songbird.federation.peers\"");
        let back: JsonRpcMethod = serde_json::from_str(&v).unwrap();
        assert_eq!(back, m);
    }

    #[test]
    fn biome_os_health_distinct_from_check() {
        assert_eq!(JsonRpcMethod::from_wire_str("health").unwrap(), JsonRpcMethod::BiomeOsHealth);
        assert_eq!(
            JsonRpcMethod::from_wire_str("health.check").unwrap(),
            JsonRpcMethod::Health(HealthMethod::Check)
        );
    }

    #[test]
    fn ipc_find_capability_roundtrip_wire_and_serde() {
        let wire = "ipc.find_capability";
        let m = JsonRpcMethod::from_wire_str(wire).unwrap();
        assert_eq!(m, JsonRpcMethod::Ipc(IpcMethod::FindCapability));
        assert_eq!(m.as_wire_str(), wire);
        assert_eq!(m.to_string(), wire);
        let v = serde_json::to_string(&m).unwrap();
        assert_eq!(v, "\"ipc.find_capability\"");
        let back: JsonRpcMethod = serde_json::from_str(&v).unwrap();
        assert_eq!(back, m);
    }

    #[test]
    fn storage_methods_roundtrip() {
        for (wire, expected) in [
            ("storage.get", JsonRpcMethod::Storage(StorageMethod::Get)),
            ("storage.put", JsonRpcMethod::Storage(StorageMethod::Put)),
            ("storage.delete", JsonRpcMethod::Storage(StorageMethod::Delete)),
            ("storage.list", JsonRpcMethod::Storage(StorageMethod::List)),
            ("storage.flush", JsonRpcMethod::Storage(StorageMethod::Flush)),
        ] {
            let m = JsonRpcMethod::from_wire_str(wire).unwrap();
            assert_eq!(m, expected);
            assert_eq!(m.as_wire_str(), wire);
            let json = serde_json::to_string(&m).unwrap();
            let back: JsonRpcMethod = serde_json::from_str(&json).unwrap();
            assert_eq!(back, m);
        }
    }

    #[test]
    fn ipc_heartbeat_roundtrip_wire_and_serde() {
        let wire = "ipc.heartbeat";
        let m = JsonRpcMethod::from_wire_str(wire).unwrap();
        assert_eq!(m, JsonRpcMethod::Ipc(IpcMethod::Heartbeat));
        assert_eq!(m.as_wire_str(), wire);
        assert_eq!(m.to_string(), wire);
        let v = serde_json::to_string(&m).unwrap();
        assert_eq!(v, "\"ipc.heartbeat\"");
        let back: JsonRpcMethod = serde_json::from_str(&v).unwrap();
        assert_eq!(back, m);
    }
}
