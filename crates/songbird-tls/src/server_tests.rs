// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::*;
use crate::codec::Encode;
use crate::error::TlsError;
use crate::messages::{ClientHello, ContentType, Extension};
use base64::engine::general_purpose;
use base64::Engine;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn sample_config() -> TlsServerConfig {
    TlsServerConfig {
        crypto_client: SecurityTlsCryptoClient::with_socket_path(
            "/tmp/songbird-tls-server-unit-test.sock".into(),
        ),
        certificate: vec![0x30, 0x82, 0x01],
        key_id: "unit-test-key".into(),
    }
}

fn config_with_crypto_path(socket_path: String) -> TlsServerConfig {
    TlsServerConfig {
        crypto_client: SecurityTlsCryptoClient::with_socket_path(socket_path),
        certificate: vec![0x30, 0x82, 0x01, 0x04, 0x05],
        key_id: "test-key-id".into(),
    }
}

/// TLS plaintext record: 5-byte header + payload (version 0x0303).
fn tls_record(content_type: u8, payload: &[u8]) -> Vec<u8> {
    let mut r = Vec::with_capacity(5 + payload.len());
    r.push(content_type);
    r.extend_from_slice(&[0x03, 0x03]);
    let len = u16::try_from(payload.len()).expect("payload fits u16");
    r.extend_from_slice(&len.to_be_bytes());
    r.extend_from_slice(payload);
    r
}

/// Encode a `ClientHello` as a TLS Handshake record payload (type + 24-bit length + body).
fn client_hello_handshake_payload(hello: &ClientHello) -> Vec<u8> {
    let mut ch_body = Vec::new();
    hello.encode(&mut ch_body).expect("encode ClientHello");
    let mut payload = Vec::with_capacity(4 + ch_body.len());
    payload.push(crate::HANDSHAKE_TYPE_CLIENT_HELLO);
    let len = u32::try_from(ch_body.len()).expect("ClientHello fits u24");
    payload.push(((len >> 16) & 0xFF) as u8);
    payload.push(((len >> 8) & 0xFF) as u8);
    payload.push((len & 0xFF) as u8);
    payload.extend_from_slice(&ch_body);
    payload
}

fn valid_client_hello() -> ClientHello {
    ClientHello::new(
        [0xAB; 32],
        vec![crate::TLS_CHACHA20_POLY1305_SHA256],
        vec![
            Extension::SupportedVersions(vec![crate::TLS_VERSION_1_3]),
            Extension::KeyShare(vec![0xCD; 32]),
        ],
    )
}

/// Mock JSON-RPC server that answers handshake crypto calls (many one-shot TCP connections).
async fn spawn_handshake_mock_crypto_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind mock crypto");
    let addr = listener.local_addr().expect("mock crypto addr");
    tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(async move {
                let mut buf = vec![0u8; 65_536];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                if n == 0 {
                    return;
                }
                let req = String::from_utf8_lossy(&buf[..n]);
                let response = if req.contains("generate_keypair") {
                    let pk = general_purpose::STANDARD.encode([9u8; 32]);
                    let sk = general_purpose::STANDARD.encode([8u8; 32]);
                    format!(
                        r#"{{"jsonrpc":"2.0","result":{{"public_key":"{pk}","secret_key":"{sk}"}},"id":1}}"#
                    )
                } else if req.contains("derive_secret") {
                    let ss = general_purpose::STANDARD.encode([7u8; 32]);
                    format!(r#"{{"jsonrpc":"2.0","result":{{"shared_secret":"{ss}"}},"id":1}}"#)
                } else {
                    let mac = general_purpose::STANDARD.encode([4u8; 32]);
                    format!(r#"{{"jsonrpc":"2.0","result":{{"mac":"{mac}"}},"id":1}}"#)
                };
                let _ = stream.write_all(response.as_bytes()).await;
                let _ = stream.shutdown().await;
            });
        }
    });
    format!("tcp:{addr}")
}

#[test]
fn tls_acceptor_new_wraps_config() {
    let cfg = sample_config();
    let acceptor = TlsAcceptor::new(TlsServerConfig {
        crypto_client: cfg.crypto_client.clone(),
        certificate: cfg.certificate.clone(),
        key_id: cfg.key_id,
    });
    let _ = acceptor;
}

#[tokio::test]
async fn accept_rejects_non_handshake_first_record() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    let acceptor = TlsAcceptor::new(sample_config());
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        acceptor.accept(stream).await
    });
    let mut client = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let rec = tls_record(ContentType::ApplicationData as u8, b"ping");
    client.write_all(&rec).await.expect("write");
    let res = server.await.expect("join");
    assert!(res.is_err(), "expected error for non-handshake first record");
}

#[tokio::test]
async fn accept_rejects_empty_handshake_payload() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    let acceptor = TlsAcceptor::new(sample_config());
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        acceptor.accept(stream).await
    });
    let mut client = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let rec = tls_record(ContentType::Handshake as u8, &[]);
    client.write_all(&rec).await.expect("write");
    let res = server.await.expect("join");
    assert!(res.is_err(), "expected error for handshake payload too short");
}

#[tokio::test]
async fn accept_rejects_non_client_hello_handshake_type() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    let acceptor = TlsAcceptor::new(sample_config());
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        acceptor.accept(stream).await
    });
    let mut client = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let payload = [crate::HANDSHAKE_TYPE_SERVER_HELLO, 0, 0, 0];
    let rec = tls_record(ContentType::Handshake as u8, &payload);
    client.write_all(&rec).await.expect("write");
    let res = server.await.expect("join");
    assert!(res.is_err(), "expected error when first handshake is not ClientHello");
}

#[tokio::test]
async fn accept_fails_when_peer_closes_before_header() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    let acceptor = TlsAcceptor::new(sample_config());
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        acceptor.accept(stream).await
    });
    drop(tokio::net::TcpStream::connect(addr).await.expect("connect"));
    let res = server.await.expect("join");
    assert!(res.is_err(), "expected I/O error when client disconnects immediately");
}

#[test]
fn tls_acceptor_new_with_various_configurations() {
    let paths = ["/tmp/a.sock", "tcp:127.0.0.1:9901", "/var/run/biomeos/crypto.sock"];
    for path in paths {
        let cfg = config_with_crypto_path(path.to_string());
        let acceptor = TlsAcceptor::new(cfg);
        let _ = acceptor;
    }

    let empty_cert = TlsAcceptor::new(TlsServerConfig {
        crypto_client: SecurityTlsCryptoClient::with_socket_path("tcp:127.0.0.1:1".into()),
        certificate: Vec::new(),
        key_id: String::new(),
    });
    let _ = empty_cert;

    let large_cert = TlsAcceptor::new(TlsServerConfig {
        crypto_client: SecurityTlsCryptoClient::with_socket_path("tcp:127.0.0.1:2".into()),
        certificate: vec![0xFF; 4096],
        key_id: "key-with-unicode-🔐".into(),
    });
    let _ = large_cert;
}

#[tokio::test]
async fn accept_rejects_malformed_client_hello_decode() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    let acceptor = TlsAcceptor::new(sample_config());
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        acceptor.accept(stream).await
    });
    let mut client = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let payload = [crate::HANDSHAKE_TYPE_CLIENT_HELLO, 0, 0, 2, 0x03, 0x03];
    let rec = tls_record(ContentType::Handshake as u8, &payload);
    client.write_all(&rec).await.expect("write");
    let res = server.await.expect("join");
    assert!(res.is_err(), "expected decode error for truncated ClientHello body");
}

#[tokio::test]
async fn accept_rejects_client_hello_missing_key_share() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    let acceptor = TlsAcceptor::new(sample_config());
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        acceptor.accept(stream).await
    });
    let hello = ClientHello::new(
        [1u8; 32],
        vec![crate::TLS_CHACHA20_POLY1305_SHA256],
        vec![Extension::SupportedVersions(vec![crate::TLS_VERSION_1_3])],
    );
    let rec = tls_record(ContentType::Handshake as u8, &client_hello_handshake_payload(&hello));
    let mut client = tokio::net::TcpStream::connect(addr).await.expect("connect");
    client.write_all(&rec).await.expect("write");
    let res = server.await.expect("join");
    match res {
        Err(TlsError::HandshakeFailure(msg)) => {
            assert!(msg.contains("key_share"), "unexpected message: {msg}");
        }
        Ok(_) => panic!("expected HandshakeFailure for missing key_share"),
        Err(e) => panic!("expected HandshakeFailure, got {:?}", std::mem::discriminant(&e)),
    }
}

#[tokio::test]
async fn accept_rejects_client_hello_missing_supported_versions() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    let acceptor = TlsAcceptor::new(sample_config());
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        acceptor.accept(stream).await
    });
    let hello = ClientHello::new(
        [2u8; 32],
        vec![crate::TLS_CHACHA20_POLY1305_SHA256],
        vec![Extension::KeyShare(vec![0xEE; 32])],
    );
    let rec = tls_record(ContentType::Handshake as u8, &client_hello_handshake_payload(&hello));
    let mut client = tokio::net::TcpStream::connect(addr).await.expect("connect");
    client.write_all(&rec).await.expect("write");
    let res = server.await.expect("join");
    assert!(res.is_err(), "expected validation error without supported_versions");
}

#[tokio::test]
async fn accept_fails_on_oversized_record_with_truncated_payload() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    let acceptor = TlsAcceptor::new(sample_config());
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        acceptor.accept(stream).await
    });
    let mut client = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let header = vec![ContentType::Handshake as u8, 0x03, 0x03, 0x10, 0x00];
    client.write_all(&header).await.expect("write header");
    drop(client);
    let res = server.await.expect("join");
    match res {
        Err(TlsError::IoError(msg)) => {
            assert!(
                msg.contains("payload") || msg.contains("read"),
                "expected read failure for truncated oversized record: {msg}"
            );
        }
        Ok(_) => panic!("expected IoError for truncated oversized record"),
        Err(e) => panic!("expected IoError, got {:?}", std::mem::discriminant(&e)),
    }
}

#[tokio::test]
async fn accept_fails_when_peer_closes_mid_payload() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    let acceptor = TlsAcceptor::new(sample_config());
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        acceptor.accept(stream).await
    });
    let hello = valid_client_hello();
    let payload = client_hello_handshake_payload(&hello);
    let mut client = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let mut header = vec![ContentType::Handshake as u8, 0x03, 0x03];
    let len = u16::try_from(payload.len()).expect("len");
    header.extend_from_slice(&len.to_be_bytes());
    client.write_all(&header).await.expect("header");
    client.write_all(&payload[..4]).await.expect("partial");
    drop(client);
    let res = server.await.expect("join");
    assert!(res.is_err(), "expected I/O error when payload truncated");
}

#[tokio::test]
async fn accept_completes_key_derivation_with_mock_crypto() {
    let crypto_path = spawn_handshake_mock_crypto_server().await;
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    let acceptor = TlsAcceptor::new(config_with_crypto_path(crypto_path));
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        acceptor.accept(stream).await
    });
    let hello = valid_client_hello();
    let rec = tls_record(ContentType::Handshake as u8, &client_hello_handshake_payload(&hello));
    let mut client = tokio::net::TcpStream::connect(addr).await.expect("connect");
    client.write_all(&rec).await.expect("write ClientHello");
    let tls_stream = server.await.expect("join").expect("handshake should succeed");
    let _ = tls_stream;
}

#[tokio::test]
async fn tls_stream_async_read_write_and_shutdown() {
    let crypto_path = spawn_handshake_mock_crypto_server().await;
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    let acceptor = TlsAcceptor::new(config_with_crypto_path(crypto_path));
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        acceptor.accept(stream).await
    });
    let hello = valid_client_hello();
    let rec = tls_record(ContentType::Handshake as u8, &client_hello_handshake_payload(&hello));
    let (client_ready_tx, client_ready_rx) = tokio::sync::oneshot::channel();
    let client = tokio::spawn(async move {
        let mut client = tokio::net::TcpStream::connect(addr).await.expect("connect");
        client.write_all(&rec).await.expect("write ClientHello");
        client_ready_rx.await.expect("handshake done signal");
        client.write_all(b"post-handshake-probe").await.expect("client probe");
    });
    let mut tls_stream = server.await.expect("join").expect("handshake");
    client_ready_tx.send(()).expect("signal client");

    let n = AsyncWriteExt::write(&mut tls_stream, b"probe-bytes").await.expect("async write");
    assert_eq!(n, b"probe-bytes".len());
    AsyncWriteExt::flush(&mut tls_stream).await.expect("flush");

    let mut buf = [0u8; 64];
    let n = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        AsyncReadExt::read(&mut tls_stream, &mut buf),
    )
    .await
    .expect("read timed out")
    .expect("async read");
    assert_eq!(&buf[..n], b"post-handshake-probe");

    tls_stream.shutdown().await.expect("shutdown");
    client.await.expect("client task");
}

#[tokio::test]
async fn accept_fails_when_crypto_provider_unreachable() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    let dead_listener = TcpListener::bind("127.0.0.1:0").await.expect("bind dead");
    let dead_addr = dead_listener.local_addr().expect("dead addr");
    drop(dead_listener);
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

    let acceptor = TlsAcceptor::new(config_with_crypto_path(format!("tcp:{dead_addr}")));
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        acceptor.accept(stream).await
    });
    let hello = valid_client_hello();
    let rec = tls_record(ContentType::Handshake as u8, &client_hello_handshake_payload(&hello));
    let mut client = tokio::net::TcpStream::connect(addr).await.expect("connect");
    client.write_all(&rec).await.expect("write");
    let res = server.await.expect("join");
    match res {
        Err(TlsError::CryptoError(msg)) => {
            assert!(msg.contains("connect") || msg.contains("Failed"), "got: {msg}");
        }
        Ok(_) => panic!("expected CryptoError when provider unreachable"),
        Err(e) => panic!("expected CryptoError, got {:?}", std::mem::discriminant(&e)),
    }
}
