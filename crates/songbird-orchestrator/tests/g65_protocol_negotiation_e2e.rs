// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::ignore_without_reason,
    reason = "integration tests"
)]

//! G65 Protocol Negotiation E2E tests for songBird.
//!
//! Verifies that songBird's IPC session handler correctly detects and responds
//! to G65 `PROTOCOLS:` negotiation requests on the primary UDS.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

fn socket_path() -> PathBuf {
    let xdg = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(xdg).join("songbird.sock")
}

/// Helper: connect to songBird's primary UDS if it's running.
/// Returns None if songBird isn't running (test is skipped).
fn try_connect() -> Option<UnixStream> {
    let path = socket_path();
    if !path.exists() {
        return None;
    }
    UnixStream::connect(&path).ok()
}

/// G65 negotiation: client sends "PROTOCOLS: tarpc,jsonrpc\n", expects "PROTOCOL: tarpc\n"
#[test]
fn g65_negotiate_tarpc_preferred() {
    let Some(mut stream) = try_connect() else {
        eprintln!("SKIP: songBird not running (socket not found)");
        return;
    };
    stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    stream.set_write_timeout(Some(Duration::from_secs(5))).unwrap();

    stream.write_all(b"PROTOCOLS: tarpc,jsonrpc\n").unwrap();
    stream.flush().unwrap();

    let mut reader = BufReader::new(&stream);
    let mut response = String::new();
    reader.read_line(&mut response).unwrap();

    assert_eq!(response.trim(), "PROTOCOL: tarpc");
}

/// G65 negotiation: client sends "PROTOCOLS: jsonrpc\n", expects "PROTOCOL: jsonrpc\n"
#[test]
fn g65_negotiate_jsonrpc_only() {
    let Some(mut stream) = try_connect() else {
        eprintln!("SKIP: songBird not running (socket not found)");
        return;
    };
    stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    stream.set_write_timeout(Some(Duration::from_secs(5))).unwrap();

    stream.write_all(b"PROTOCOLS: jsonrpc\n").unwrap();
    stream.flush().unwrap();

    let mut reader = BufReader::new(&stream);
    let mut response = String::new();
    reader.read_line(&mut response).unwrap();

    assert_eq!(response.trim(), "PROTOCOL: jsonrpc");
}

/// Backward compatibility: plain JSON-RPC request without negotiation still works.
#[test]
fn g65_backward_compat_plain_jsonrpc() {
    let Some(mut stream) = try_connect() else {
        eprintln!("SKIP: songBird not running (socket not found)");
        return;
    };
    stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    stream.set_write_timeout(Some(Duration::from_secs(5))).unwrap();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "health.liveness",
        "id": 1
    });
    let mut wire = serde_json::to_string(&request).unwrap();
    wire.push('\n');

    stream.write_all(wire.as_bytes()).unwrap();
    stream.flush().unwrap();

    let mut reader = BufReader::new(&stream);
    let mut response = String::new();
    reader.read_line(&mut response).unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&response).unwrap();
    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["id"], 1);
}

/// Unit-level test of the negotiation types (no live server needed).
#[test]
fn g65_types_unit_tests() {
    use songbird_universal::protocol_negotiation::*;

    let req = NegotiationRequest::prefer_tarpc();
    assert_eq!(req.to_wire(), "PROTOCOLS: tarpc,jsonrpc\n");

    let parsed = NegotiationRequest::from_wire("PROTOCOLS: tarpc,jsonrpc\n").unwrap();
    assert_eq!(parsed.supported, vec![IpcProtocol::Tarpc, IpcProtocol::JsonRpc]);

    let resp = NegotiationResponse::new(IpcProtocol::Tarpc);
    assert_eq!(resp.to_wire(), "PROTOCOL: tarpc\n");

    let selected = select_protocol(
        &[IpcProtocol::Tarpc, IpcProtocol::JsonRpc],
        &IpcProtocol::all_supported(),
    );
    assert_eq!(selected, IpcProtocol::Tarpc);
}

/// Full client-server negotiation via tokio duplex (no live server needed).
#[tokio::test]
async fn g65_negotiate_full_duplex() {
    use songbird_universal::protocol_negotiation::*;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, duplex};

    let (mut client, mut server) = duplex(1024);

    let server_handle = tokio::spawn(async move {
        let mut reader = BufReader::new(&mut server);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let request = NegotiationRequest::from_wire(&line).unwrap();
        let selected = select_protocol(&request.supported, &IpcProtocol::all_supported());
        let response = NegotiationResponse::new(selected);
        reader
            .get_mut()
            .write_all(response.to_wire().as_bytes())
            .await
            .unwrap();
        reader.get_mut().flush().await.unwrap();
        selected
    });

    let result = negotiate_client(
        &mut client,
        &[IpcProtocol::Tarpc, IpcProtocol::JsonRpc],
    )
    .await
    .unwrap();

    assert_eq!(result, IpcProtocol::Tarpc);
    assert_eq!(server_handle.await.unwrap(), IpcProtocol::Tarpc);
}
