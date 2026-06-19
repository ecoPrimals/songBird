// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![forbid(unsafe_code)]

use super::super::IpcServiceHandler;
use serde_json::Value;
use songbird_types::json_rpc_method::{
    BirdsongMethod, FederationMethod, IgdMethod, JsonRpcMethod, OnionMethod, PunchMethod,
    RelayMethod, StunMethod, TorMethod,
};

#[expect(clippy::too_many_lines, reason = "network domain dispatch table")]
pub(super) async fn dispatch_network(
    handler: &IpcServiceHandler,
    method: JsonRpcMethod,
    params: Value,
) -> Result<Value, String> {
    match method {
        JsonRpcMethod::Stun(StunMethod::Serve) => handler.stun_handler.handle_serve(params).await,
        JsonRpcMethod::Stun(StunMethod::Stop) => handler.stun_handler.handle_stop(params).await,
        JsonRpcMethod::Stun(StunMethod::Status) => handler.stun_handler.handle_status(params).await,
        JsonRpcMethod::Stun(StunMethod::GetPublicAddress) => {
            handler.stun_handler.handle_get_public_address(params).await
        }
        JsonRpcMethod::Stun(StunMethod::Bind) => handler.stun_handler.handle_bind(params).await,
        JsonRpcMethod::Stun(StunMethod::ProbePortPattern) => {
            handler.stun_handler.handle_probe_port_pattern(params).await
        }
        JsonRpcMethod::Stun(StunMethod::DetectNatType) => {
            handler.stun_handler.handle_detect_nat_type(params).await
        }

        JsonRpcMethod::Igd(IgdMethod::Discover) => {
            Ok(handler.igd_handler.handle_discover(params).await)
        }
        JsonRpcMethod::Igd(IgdMethod::MapPort) => {
            Ok(handler.igd_handler.handle_map_port(params).await)
        }
        JsonRpcMethod::Igd(IgdMethod::UnmapPort) => {
            Ok(handler.igd_handler.handle_unmap_port(params).await)
        }
        JsonRpcMethod::Igd(IgdMethod::Status) => {
            Ok(handler.igd_handler.handle_status(params).await)
        }
        JsonRpcMethod::Igd(IgdMethod::ExternalIp) => {
            Ok(handler.igd_handler.handle_external_ip(params).await)
        }
        JsonRpcMethod::Igd(IgdMethod::AutoConfigure) => {
            Ok(handler.igd_handler.handle_auto_configure(params).await)
        }

        JsonRpcMethod::Relay(RelayMethod::Serve) => {
            handler.relay_handler.handle_serve(params).await
        }
        JsonRpcMethod::Relay(RelayMethod::Stop) => handler.relay_handler.handle_stop(params).await,
        JsonRpcMethod::Relay(RelayMethod::Status) => {
            handler.relay_handler.handle_status(params).await
        }
        JsonRpcMethod::Relay(RelayMethod::Allocate) => {
            handler.relay_handler.handle_allocate(params).await
        }

        JsonRpcMethod::Birdsong(BirdsongMethod::GenerateEncryptedBeacon) => {
            handler.birdsong_handler.handle_generate_encrypted_beacon(params).await
        }
        JsonRpcMethod::Birdsong(BirdsongMethod::DecryptBeacon) => {
            handler.birdsong_handler.handle_decrypt_beacon(params).await
        }
        JsonRpcMethod::Birdsong(BirdsongMethod::VerifyLineage) => {
            handler.birdsong_handler.handle_verify_lineage(params).await
        }
        JsonRpcMethod::Birdsong(BirdsongMethod::GetLineage) => {
            handler.birdsong_handler.handle_get_lineage(params).await
        }
        JsonRpcMethod::Birdsong(BirdsongMethod::Advertise) => {
            handler.handle_birdsong_advertise(params).await
        }
        JsonRpcMethod::Birdsong(BirdsongMethod::Schema) => {
            handler.birdsong_handler.handle_schema(params).await
        }

        JsonRpcMethod::Punch(PunchMethod::Request) => {
            handler.punch_handler.handle_request(params).await
        }
        JsonRpcMethod::Punch(PunchMethod::Coordinate) => {
            handler.punch_handler.handle_coordinate(params).await
        }
        JsonRpcMethod::Punch(PunchMethod::Status) => {
            handler.punch_handler.handle_status(params).await
        }

        JsonRpcMethod::Onion(OnionMethod::Start) => {
            handler.onion_handler.handle_start(params).await
        }
        JsonRpcMethod::Onion(OnionMethod::Stop) => handler.onion_handler.handle_stop(params).await,
        JsonRpcMethod::Onion(OnionMethod::Status) => {
            handler.onion_handler.handle_status(params).await
        }
        JsonRpcMethod::Onion(OnionMethod::Connect) => {
            handler.onion_handler.handle_connect(params).await
        }
        JsonRpcMethod::Onion(OnionMethod::Address) => {
            handler.onion_handler.handle_address(params).await
        }

        JsonRpcMethod::Federation(FederationMethod::Peers) => {
            handler.handle_federation_peers_rpc().await
        }
        JsonRpcMethod::Federation(FederationMethod::Status) => {
            handler.handle_federation_status_rpc().await
        }

        JsonRpcMethod::Tor(TorMethod::Status) => handler.tor_handler.handle_status(params).await,
        JsonRpcMethod::Tor(TorMethod::Connect) => handler.tor_handler.handle_connect(params).await,
        JsonRpcMethod::Tor(TorMethod::ServiceStart) => {
            handler.tor_handler.handle_service_start(params).await
        }
        JsonRpcMethod::Tor(TorMethod::ServiceStop) => {
            handler.tor_handler.handle_service_stop(params).await
        }
        JsonRpcMethod::Tor(TorMethod::ConsensusFetch) => {
            handler.tor_handler.handle_consensus_fetch(params).await
        }
        JsonRpcMethod::Tor(TorMethod::CircuitBuild) => {
            handler.tor_handler.handle_circuit_build(params).await
        }
        JsonRpcMethod::Tor(TorMethod::CircuitClose) => {
            handler.tor_handler.handle_circuit_close(params).await
        }

        other => Err(format!("Unknown method: {other}")),
    }
}
