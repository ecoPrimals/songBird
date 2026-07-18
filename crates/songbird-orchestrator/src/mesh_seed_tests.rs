// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for mesh seed peer parsing, `WireGuard` extraction, and mesh population.

#![allow(clippy::expect_used, clippy::unwrap_used, reason = "test assertions")]

use super::*;

#[test]
fn parse_empty_string() {
    assert!(parse_peers_str("").is_empty());
}

#[test]
fn parse_valid_entries() {
    let peers = parse_peers_str("iron-gate@192.168.1.238:7700,south-gate@192.168.4.29:7700");
    assert_eq!(peers.len(), 2);
    assert_eq!(peers[0], (String::from("iron-gate"), String::from("192.168.1.238:7700")));
    assert_eq!(peers[1], (String::from("south-gate"), String::from("192.168.4.29:7700")));
}

#[test]
fn parse_skips_invalid() {
    let peers =
        parse_peers_str("good@192.168.1.1:7700,bad-no-at-sign,missing@not-a-port,,@empty:0");
    assert_eq!(peers.len(), 1);
    assert_eq!(peers[0].0, "good");
}

#[test]
fn parse_handles_whitespace() {
    let peers = parse_peers_str(" east@10.0.0.1:7700 , west@10.0.0.2:7700 ");
    assert_eq!(peers.len(), 2);
    assert_eq!(peers[0].0, "east");
    assert_eq!(peers[1].0, "west");
}

#[test]
fn parse_address_only_format() {
    let peers = parse_peers_str("192.168.1.144:7700,192.168.1.238:7700");
    assert_eq!(peers.len(), 2);
    assert_eq!(peers[0], (String::from("peer-192.168.1.144"), String::from("192.168.1.144:7700")));
    assert_eq!(peers[1], (String::from("peer-192.168.1.238"), String::from("192.168.1.238:7700")));
}

#[test]
fn parse_mixed_formats() {
    let peers =
        parse_peers_str("iron-gate@192.168.1.238:7700,192.168.4.29:7700,south@10.0.0.1:7700");
    assert_eq!(peers.len(), 3);
    assert_eq!(peers[0].0, "iron-gate");
    assert_eq!(peers[1].0, "peer-192.168.4.29");
    assert_eq!(peers[2].0, "south");
}

#[tokio::test]
async fn spawn_mesh_seed_populates_mesh() {
    let _guard = crate::test_sync_env::env_lock();
    songbird_process_env::set_var("SONGBIRD_NODE_ID", "test-gate-seed");
    songbird_process_env::set_var(
        "SONGBIRD_PEERS",
        "iron-gate@192.168.1.238:7700,south-gate@192.168.4.29:7700",
    );

    let mesh_handler = Arc::new(MeshHandler::new());
    spawn_mesh_seed(Arc::clone(&mesh_handler));

    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        let guard = mesh_handler.mesh().await;
        if let Some(mesh) = guard.as_ref() {
            let reachable = mesh.get_reachable_nodes().await;
            if reachable.len() >= 2 {
                break;
            }
        }
        drop(guard);
        assert!(tokio::time::Instant::now() < deadline, "mesh not populated within 2s");
        tokio::task::yield_now().await;
    }

    let guard = mesh_handler.mesh().await;
    let mesh = guard.as_ref().expect("mesh should be initialized");
    let reachable = mesh.get_reachable_nodes().await;

    songbird_process_env::remove_var("SONGBIRD_PEERS");
    songbird_process_env::remove_var("SONGBIRD_NODE_ID");

    assert_eq!(reachable.len(), 2);
    assert!(reachable.contains(&String::from("iron-gate")));
    assert!(reachable.contains(&String::from("south-gate")));
}

#[tokio::test]
async fn spawn_mesh_seed_registers_overlay_peers() {
    let _guard = crate::test_sync_env::env_lock();
    songbird_process_env::set_var("SONGBIRD_NODE_ID", "east-gate-overlay-test");
    songbird_process_env::set_var(
        "SONGBIRD_PEERS",
        "flock-gate@203.0.113.50:7700,golgi@203.0.113.51:7700",
    );
    songbird_process_env::set_var(
        "SONGBIRD_OVERLAY_PEERS",
        "flock-gate@10.13.37.6:7700,golgi@10.13.37.1:7700",
    );

    let mesh_handler = Arc::new(MeshHandler::new());
    spawn_mesh_seed(Arc::clone(&mesh_handler));

    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        let guard = mesh_handler.mesh().await;
        if let Some(mesh) = guard.as_ref() {
            let reachable = mesh.get_reachable_nodes().await;
            if reachable.len() >= 2 {
                break;
            }
        }
        drop(guard);
        assert!(tokio::time::Instant::now() < deadline, "mesh not populated within 2s");
        tokio::task::yield_now().await;
    }

    let guard = mesh_handler.mesh().await;
    let mesh = guard.as_ref().expect("mesh should be initialized");

    let best = mesh.get_best_path("flock-gate").await;
    assert!(best.is_some(), "flock-gate should have a path");
    let best = best.unwrap();
    assert!(
        matches!(best.endpoint_type, songbird_onion_relay::mesh::EndpointType::Overlay { .. }),
        "Expected Overlay as best path, got {:?}",
        best.endpoint_type
    );

    songbird_process_env::remove_var("SONGBIRD_PEERS");
    songbird_process_env::remove_var("SONGBIRD_OVERLAY_PEERS");
    songbird_process_env::remove_var("SONGBIRD_NODE_ID");
}

#[test]
fn overlay_peers_parsed_same_format_as_regular() {
    let overlay = parse_peers_str("flock@10.13.37.6:7700,golgi@10.13.37.1:7700");
    assert_eq!(overlay.len(), 2);
    assert_eq!(overlay[0], (String::from("flock"), String::from("10.13.37.6:7700")));
    assert_eq!(overlay[1], (String::from("golgi"), String::from("10.13.37.1:7700")));
}

#[test]
fn parse_wg_dump_extracts_overlay_peers() {
    let dump = "\
wg0\tOURPUBKEY1234567890abcdef=\tOURPRIVKEY1234567890abcdef=\t51820\toff\n\
wg0\tABCDEFGH12345678pubkey1=\t(none)\t10.13.37.1:51820\t10.13.37.1/32\t1719043200\t12345\t67890\t25\n\
wg0\tIJKLMNOP87654321pubkey2=\t(none)\t10.13.37.5:51820\t10.13.37.5/32\t1719043200\t23456\t78901\t25\n\
wg0\tQRSTUVWX11111111pubkey3=\t(none)\t203.0.113.50:51820\t10.13.37.6/32\t1719043200\t34567\t89012\t25";

    let peers = parse_wg_dump(dump, "10.13.37");
    assert_eq!(peers.len(), 3);
    assert_eq!(peers[0].0, "wg-ABCDEFGH");
    assert_eq!(peers[0].1, "10.13.37.1:7700");
    assert_eq!(peers[1].0, "wg-IJKLMNOP");
    assert_eq!(peers[1].1, "10.13.37.5:7700");
    assert_eq!(peers[2].0, "wg-QRSTUVWX");
    assert_eq!(peers[2].1, "10.13.37.6:7700");
}

#[test]
fn parse_wg_dump_skips_non_matching_subnet() {
    let dump = "\
wg0\tOURPUBKEY=\tPRIVKEY=\t51820\toff\n\
wg0\tPEERKEY1234=\t(none)\t192.168.1.50:51820\t192.168.1.50/32\t1719043200\t100\t200\t25";

    let peers = parse_wg_dump(dump, "10.13.37");
    assert!(peers.is_empty());
}

#[test]
fn parse_wg_dump_empty_output() {
    let peers = parse_wg_dump("", "10.13.37");
    assert!(peers.is_empty());
}

#[test]
fn parse_wg_dump_handles_multiple_allowed_ips() {
    let dump = "\
wg0\tOUR=\tPRIV=\t51820\toff\n\
wg0\tMULTIPEER1234567=\t(none)\t1.2.3.4:51820\t192.168.0.0/24,10.13.37.2/32\t0\t0\t0\t25";

    let peers = parse_wg_dump(dump, "10.13.37");
    assert_eq!(peers.len(), 1);
    assert_eq!(peers[0].0, "wg-MULTIPEE");
    assert_eq!(peers[0].1, "10.13.37.2:7700");
}

#[test]
fn parse_wg_conf_extracts_peers() {
    let conf = "\
[Interface]
PrivateKey = aGVsbG8gd29ybGQ=
Address = 10.13.37.6/24
ListenPort = 51820

[Peer]
PublicKey = ABCDEFGH12345678pubkey1=
AllowedIPs = 10.13.37.1/32
Endpoint = 203.0.113.1:51820

[Peer]
PublicKey = IJKLMNOP87654321pubkey2=
AllowedIPs = 10.13.37.5/32
Endpoint = 203.0.113.5:51820

[Peer]
PublicKey = QRSTUVWX11111111pubkey3=
AllowedIPs = 10.13.37.2/32
Endpoint = 192.168.4.3:51820
";

    let peers = parse_wg_conf(conf, "10.13.37");
    assert_eq!(peers.len(), 3);
    assert_eq!(peers[0].0, "wg-ABCDEFGH");
    assert_eq!(peers[0].1, "10.13.37.1:7700");
    assert_eq!(peers[1].0, "wg-IJKLMNOP");
    assert_eq!(peers[1].1, "10.13.37.5:7700");
    assert_eq!(peers[2].0, "wg-QRSTUVWX");
    assert_eq!(peers[2].1, "10.13.37.2:7700");
}

#[test]
fn parse_wg_conf_skips_non_overlay_peers() {
    let conf = "\
[Interface]
PrivateKey = key=
Address = 10.13.37.6/24

[Peer]
PublicKey = NOTMATCH12345678=
AllowedIPs = 192.168.1.0/24
Endpoint = 1.2.3.4:51820
";

    let peers = parse_wg_conf(conf, "10.13.37");
    assert!(peers.is_empty());
}

#[test]
fn parse_wg_conf_handles_multiple_allowed_ips() {
    let conf = "\
[Interface]
PrivateKey = key=
Address = 10.13.37.6/24

[Peer]
PublicKey = MULTIIP12345=
AllowedIPs = 192.168.0.0/16, 10.13.37.7/32
Endpoint = 5.6.7.8:51820
";

    let peers = parse_wg_conf(conf, "10.13.37");
    assert_eq!(peers.len(), 1);
    assert_eq!(peers[0].0, "wg-MULTIIP1");
    assert_eq!(peers[0].1, "10.13.37.7:7700");
}

#[test]
fn parse_wg_conf_empty_input() {
    let peers = parse_wg_conf("", "10.13.37");
    assert!(peers.is_empty());
}

#[test]
fn mesh_peers_toml_format_parses() {
    let content = r#"
[[peers]]
node_id = "east-gate"
address = "10.13.37.5:8080"

[[peers]]
node_id = "golgi"
address = "10.13.37.1:8080"
"#;
    let parsed: MeshPeersFile = toml::from_str(content).unwrap();
    assert_eq!(parsed.peers.len(), 2);
    assert_eq!(parsed.peers[0].node_id, "east-gate");
    assert_eq!(parsed.peers[0].address, "10.13.37.5:8080");
    assert_eq!(parsed.peers[1].node_id, "golgi");
    assert_eq!(parsed.peers[1].address, "10.13.37.1:8080");
}

#[test]
fn mesh_peers_toml_skips_empty_entries() {
    let content = r#"
[[peers]]
node_id = ""
address = "10.13.37.5:8080"

[[peers]]
node_id = "valid"
address = "10.13.37.1:8080"
"#;
    let parsed: MeshPeersFile = toml::from_str(content).unwrap();
    let filtered: Vec<_> = parsed
        .peers
        .into_iter()
        .filter(|p| !p.node_id.is_empty() && !p.address.is_empty())
        .collect();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].node_id, "valid");
}
