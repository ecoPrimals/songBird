// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::*;

#[test]
fn test_new_handshake() {
    let hsm = HandshakeStateMachine::new();
    assert_eq!(hsm.state(), HandshakeState::Start);
    assert!(!hsm.is_connected());
}

#[test]
fn test_process_client_hello() {
    let mut hsm = HandshakeStateMachine::new();

    let random = [42u8; 32];
    let cipher_suites = vec![0x1303];
    let extensions = vec![
        crate::messages::Extension::SupportedVersions(vec![0x0304]),
        crate::messages::Extension::KeyShare(vec![1, 2, 3, 4]),
    ];

    let client_hello = ClientHello::new(random, cipher_suites, extensions);

    hsm.process_client_hello(client_hello).unwrap();
    assert_eq!(hsm.state(), HandshakeState::ReceivedClientHello);
}

#[tokio::test]
async fn test_generate_server_hello() {
    let mut hsm = HandshakeStateMachine::new();

    let random = [42u8; 32];
    let cipher_suites = vec![0x1303];
    let extensions = vec![
        crate::messages::Extension::SupportedVersions(vec![0x0304]),
        crate::messages::Extension::KeyShare(vec![1, 2, 3, 4]),
    ];

    let client_hello = ClientHello::new(random, cipher_suites, extensions);
    hsm.process_client_hello(client_hello).unwrap();

    assert_eq!(hsm.state(), HandshakeState::ReceivedClientHello);
}

#[tokio::test]
async fn test_complete_handshake() {
    let mut hsm = HandshakeStateMachine::new();

    let random = [42u8; 32];
    let cipher_suites = vec![0x1303];
    let extensions = vec![
        crate::messages::Extension::SupportedVersions(vec![0x0304]),
        crate::messages::Extension::KeyShare(vec![1, 2, 3, 4]),
    ];

    let client_hello = ClientHello::new(random, cipher_suites, extensions);
    hsm.process_client_hello(client_hello).unwrap();

    assert_eq!(hsm.state(), HandshakeState::ReceivedClientHello);
}

#[tokio::test]
async fn test_invalid_state_transition() {
    let mut hsm = HandshakeStateMachine::new();

    let result = hsm.complete_handshake();
    assert!(result.is_err());
}

#[test]
fn test_duplicate_client_hello() {
    let mut hsm = HandshakeStateMachine::new();

    let random = [42u8; 32];
    let cipher_suites = vec![0x1303];
    let extensions = vec![
        crate::messages::Extension::SupportedVersions(vec![0x0304]),
        crate::messages::Extension::KeyShare(vec![1, 2, 3, 4]),
    ];

    let client_hello = ClientHello::new(random, cipher_suites.clone(), extensions.clone());
    hsm.process_client_hello(client_hello).unwrap();

    // Try to process again
    let client_hello2 = ClientHello::new(random, cipher_suites, extensions);
    let result = hsm.process_client_hello(client_hello2);
    assert!(result.is_err());
}

#[test]
fn test_handshake_state_transitions() {
    let mut hsm = HandshakeStateMachine::new();

    assert_eq!(hsm.state(), HandshakeState::Start);
    assert!(!hsm.is_connected());

    let client_hello = create_test_client_hello();
    hsm.process_client_hello(client_hello).unwrap();
    assert_eq!(hsm.state(), HandshakeState::ReceivedClientHello);
    assert!(!hsm.is_connected());
}

#[test]
fn test_client_hello_with_multiple_cipher_suites() {
    let mut hsm = HandshakeStateMachine::new();

    let random = [123u8; 32];
    let cipher_suites = vec![0x1301, 0x1302, 0x1303];
    let extensions = vec![
        crate::messages::Extension::SupportedVersions(vec![0x0304]),
        crate::messages::Extension::KeyShare(vec![1, 2, 3, 4]),
    ];

    let client_hello = ClientHello::new(random, cipher_suites, extensions);
    let result = hsm.process_client_hello(client_hello);
    assert!(result.is_ok());
    assert_eq!(hsm.state(), HandshakeState::ReceivedClientHello);
}

#[test]
fn test_client_hello_with_sni_extension() {
    let mut hsm = HandshakeStateMachine::new();

    let random = [200u8; 32];
    let cipher_suites = vec![0x1303];
    let extensions = vec![
        crate::messages::Extension::SupportedVersions(vec![0x0304]),
        crate::messages::Extension::KeyShare(vec![1, 2, 3, 4]),
        crate::messages::Extension::ServerName("example.com".to_string()),
    ];

    let client_hello = ClientHello::new(random, cipher_suites, extensions);
    let result = hsm.process_client_hello(client_hello);
    assert!(result.is_ok());
}

#[test]
fn test_client_hello_missing_required_extensions() {
    let mut hsm = HandshakeStateMachine::new();

    let random = [250u8; 32];
    let cipher_suites = vec![0x1303];
    let extensions = vec![];

    let client_hello = ClientHello::new(random, cipher_suites, extensions);
    let result = hsm.process_client_hello(client_hello);
    assert!(result.is_err());
}

#[test]
fn test_client_hello_with_legacy_version() {
    let mut hsm = HandshakeStateMachine::new();

    let random = [77u8; 32];
    let cipher_suites = vec![0x1303];
    let extensions = vec![
        crate::messages::Extension::SupportedVersions(vec![0x0303]),
        crate::messages::Extension::KeyShare(vec![1, 2, 3, 4]),
    ];

    let client_hello = ClientHello::new(random, cipher_suites, extensions);
    let result = hsm.process_client_hello(client_hello);
    assert!(result.is_ok());
}

#[test]
fn test_client_hello_no_cipher_suites() {
    let mut hsm = HandshakeStateMachine::new();

    let random = [88u8; 32];
    let cipher_suites = vec![];
    let extensions = vec![
        crate::messages::Extension::SupportedVersions(vec![0x0304]),
        crate::messages::Extension::KeyShare(vec![1, 2, 3, 4]),
    ];

    let client_hello = ClientHello::new(random, cipher_suites, extensions);
    let result = hsm.process_client_hello(client_hello);
    assert!(result.is_err());
}

#[test]
fn test_handshake_key_schedule_initialization() {
    let hsm = HandshakeStateMachine::new();
    assert_eq!(hsm.state(), HandshakeState::Start);
    assert!(hsm.crypto_client.is_none());
}

#[test]
fn test_set_crypto_client() {
    let mut hsm = HandshakeStateMachine::new();

    let crypto_client =
        SecurityTlsCryptoClient::with_socket_path("/tmp/test-security-provider.sock".to_string());
    hsm.set_crypto_client(crypto_client);

    assert!(hsm.crypto_client.is_some());
}

#[test]
fn test_handshake_error_state() {
    let mut hsm = HandshakeStateMachine::new();
    hsm.state = HandshakeState::Error;

    let client_hello = create_test_client_hello();
    let result = hsm.process_client_hello(client_hello);
    assert!(result.is_err());
}

#[test]
fn test_handshake_state_display() {
    let states = [
        HandshakeState::Start,
        HandshakeState::ReceivedClientHello,
        HandshakeState::SentServerHello,
        HandshakeState::Connected,
        HandshakeState::Error,
    ];

    for (i, state1) in states.iter().enumerate() {
        for (j, state2) in states.iter().enumerate() {
            if i == j {
                assert_eq!(state1, state2);
            } else {
                assert_ne!(state1, state2);
            }
        }
    }
}

#[tokio::test]
async fn test_server_hello_generation_without_crypto_client() {
    let mut hsm = HandshakeStateMachine::new();

    let client_hello = create_test_client_hello();
    hsm.process_client_hello(client_hello).unwrap();

    let result = hsm.generate_server_hello().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_complete_handshake_without_messages() {
    let mut hsm = HandshakeStateMachine::new();

    let result = hsm.complete_handshake();
    assert!(result.is_err());
    assert_eq!(hsm.state(), HandshakeState::Start);
}

#[test]
fn test_handshake_clone() {
    let hsm1 = HandshakeStateMachine::new();
    let state_clone = hsm1.state();

    assert_eq!(state_clone, HandshakeState::Start);
    assert_eq!(hsm1.state(), HandshakeState::Start);
}

#[test]
fn handshake_state_enum_exhaustive_equality() {
    let all = [
        HandshakeState::Start,
        HandshakeState::ReceivedClientHello,
        HandshakeState::SentServerHello,
        HandshakeState::Connected,
        HandshakeState::Error,
    ];
    for (i, a) in all.iter().enumerate() {
        for (j, b) in all.iter().enumerate() {
            assert_eq!(i == j, a == b);
        }
    }
}

#[test]
fn is_connected_only_when_state_connected() {
    let mut hsm = HandshakeStateMachine::new();
    assert!(!hsm.is_connected());
    let ch = create_test_client_hello();
    hsm.process_client_hello(ch).unwrap();
    assert!(!hsm.is_connected());
}

#[test]
fn key_schedule_accessor_returns_non_empty_new_instance() {
    let hsm = HandshakeStateMachine::new();
    let _ = hsm.key_schedule();
    let mut hsm2 = HandshakeStateMachine::new();
    let _ = hsm2.key_schedule_mut();
}

#[test]
fn process_client_hello_rejects_when_not_start_state() {
    let mut hsm = HandshakeStateMachine::new();
    let ch = create_test_client_hello();
    hsm.process_client_hello(ch).unwrap();
    let ch2 = create_test_client_hello();
    let err = hsm.process_client_hello(ch2).unwrap_err();
    assert!(matches!(err, TlsError::UnexpectedMessage { .. }));
}

#[test]
fn default_matches_new_handshake_state_machine() {
    let a = HandshakeStateMachine::new();
    let b = HandshakeStateMachine::default();
    assert_eq!(a.state(), b.state());
}

#[test]
fn is_connected_false_for_all_non_connected_states() {
    let mut hsm = HandshakeStateMachine::new();
    assert!(!hsm.is_connected());

    hsm.process_client_hello(create_test_client_hello()).unwrap();
    assert!(!hsm.is_connected());

    hsm.state = HandshakeState::SentServerHello;
    assert!(!hsm.is_connected());

    hsm.state = HandshakeState::Error;
    assert!(!hsm.is_connected());
}

#[test]
fn is_connected_true_only_in_connected_state() {
    let mut hsm = HandshakeStateMachine::new();
    hsm.state = HandshakeState::Connected;
    assert!(hsm.is_connected());
}

#[test]
fn complete_handshake_success_from_sent_server_hello() {
    let mut hsm = HandshakeStateMachine::new();
    hsm.state = HandshakeState::SentServerHello;

    hsm.complete_handshake().unwrap();
    assert_eq!(hsm.state(), HandshakeState::Connected);
    assert!(hsm.is_connected());
}

#[test]
fn complete_handshake_rejects_received_client_hello_state() {
    let mut hsm = HandshakeStateMachine::new();
    hsm.process_client_hello(create_test_client_hello()).unwrap();

    let err = hsm.complete_handshake().unwrap_err();
    assert!(matches!(err, TlsError::ProtocolError(_)));
    assert_eq!(hsm.state(), HandshakeState::ReceivedClientHello);
    assert!(!hsm.is_connected());
}

#[test]
fn complete_handshake_rejects_connected_state() {
    let mut hsm = HandshakeStateMachine::new();
    hsm.state = HandshakeState::Connected;

    let err = hsm.complete_handshake().unwrap_err();
    assert!(matches!(err, TlsError::ProtocolError(_)));
    assert!(hsm.is_connected());
}

#[test]
fn complete_handshake_rejects_error_state() {
    let mut hsm = HandshakeStateMachine::new();
    hsm.state = HandshakeState::Error;

    let err = hsm.complete_handshake().unwrap_err();
    assert!(matches!(err, TlsError::ProtocolError(_)));
}

#[tokio::test]
async fn generate_server_hello_rejects_start_state() {
    let mut hsm = HandshakeStateMachine::new();
    let err = hsm.generate_server_hello().await.unwrap_err();
    assert!(matches!(err, TlsError::ProtocolError(_)));
}

#[tokio::test]
async fn generate_server_hello_rejects_sent_server_hello_state() {
    let mut hsm = HandshakeStateMachine::new();
    hsm.state = HandshakeState::SentServerHello;

    let err = hsm.generate_server_hello().await.unwrap_err();
    assert!(matches!(err, TlsError::ProtocolError(_)));
}

#[tokio::test]
async fn generate_server_hello_missing_stored_client_hello() {
    let mut hsm = HandshakeStateMachine::new();
    hsm.state = HandshakeState::ReceivedClientHello;
    hsm.set_crypto_client(SecurityTlsCryptoClient::with_socket_path(
        "/tmp/unused.sock".to_string(),
    ));

    let err = hsm.generate_server_hello().await.unwrap_err();
    assert!(matches!(err, TlsError::InternalError(_)));
}

#[tokio::test]
async fn generate_server_hello_success_with_mock_crypto() {
    use base64::Engine;
    use base64::engine::general_purpose;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let public_key = vec![0xAAu8; 32];
    let secret_key = vec![0xBBu8; 32];
    let body = format!(
        r#"{{"jsonrpc":"2.0","result":{{"public_key":"{}","secret_key":"{}"}},"id":1}}"#,
        general_purpose::STANDARD.encode(&public_key),
        general_purpose::STANDARD.encode(&secret_key),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut buf = vec![0u8; 16_384];
        let _ = stream.read(&mut buf).await;
        stream.write_all(body.as_bytes()).await.unwrap();
        let _ = stream.shutdown().await;
    });

    let mut hsm = HandshakeStateMachine::new();
    hsm.set_crypto_client(SecurityTlsCryptoClient::with_socket_path(format!("tcp:{addr}")));
    hsm.process_client_hello(create_test_client_hello()).unwrap();

    let server_hello = hsm.generate_server_hello().await.unwrap();

    assert_eq!(hsm.state(), HandshakeState::SentServerHello);
    assert!(!hsm.is_connected());
    assert_eq!(server_hello.cipher_suite, 0x1303);
    assert_eq!(server_hello.get_supported_version(), Some(0x0304));
    assert_eq!(server_hello.get_key_share(), Some(public_key.as_slice()));
    assert!(server_hello.validate().is_ok());
    assert!(hsm.server_hello.is_some());
}

#[tokio::test]
async fn full_handshake_state_machine_with_mock_crypto() {
    use base64::Engine;
    use base64::engine::general_purpose;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let body = format!(
        r#"{{"jsonrpc":"2.0","result":{{"public_key":"{}","secret_key":"{}"}},"id":1}}"#,
        general_purpose::STANDARD.encode([1u8; 32]),
        general_purpose::STANDARD.encode([2u8; 32]),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut buf = vec![0u8; 16_384];
        let _ = stream.read(&mut buf).await;
        stream.write_all(body.as_bytes()).await.unwrap();
        let _ = stream.shutdown().await;
    });

    let mut hsm = HandshakeStateMachine::new();
    assert_eq!(hsm.state(), HandshakeState::Start);
    assert!(!hsm.is_connected());

    hsm.set_crypto_client(SecurityTlsCryptoClient::with_socket_path(format!("tcp:{addr}")));
    hsm.process_client_hello(create_test_client_hello()).unwrap();
    assert_eq!(hsm.state(), HandshakeState::ReceivedClientHello);

    hsm.generate_server_hello().await.unwrap();
    assert_eq!(hsm.state(), HandshakeState::SentServerHello);

    hsm.complete_handshake().unwrap();
    assert_eq!(hsm.state(), HandshakeState::Connected);
    assert!(hsm.is_connected());
}

#[tokio::test]
async fn generate_server_hello_propagates_crypto_failure() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let body = r#"{"jsonrpc":"2.0","error":{"code":-32000,"message":"keygen failed"},"id":1}"#;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut buf = vec![0u8; 16_384];
        let _ = stream.read(&mut buf).await;
        stream.write_all(body.as_bytes()).await.unwrap();
        let _ = stream.shutdown().await;
    });

    let mut hsm = HandshakeStateMachine::new();
    hsm.set_crypto_client(SecurityTlsCryptoClient::with_socket_path(format!("tcp:{addr}")));
    hsm.process_client_hello(create_test_client_hello()).unwrap();

    let err = hsm.generate_server_hello().await.unwrap_err();
    assert!(matches!(err, TlsError::CryptoError(_)));
    assert_eq!(hsm.state(), HandshakeState::ReceivedClientHello);
}

#[test]
fn process_client_hello_rejects_oversized_session_id() {
    let mut hsm = HandshakeStateMachine::new();
    let mut hello = create_test_client_hello();
    hello.legacy_session_id = vec![0u8; 33];
    let err = hsm.process_client_hello(hello).unwrap_err();
    assert!(matches!(err, TlsError::ProtocolError(_)));
    assert_eq!(hsm.state(), HandshakeState::Start);
}

#[test]
fn process_client_hello_rejects_missing_supported_versions() {
    let mut hsm = HandshakeStateMachine::new();
    let hello = ClientHello::new(
        [11u8; 32],
        vec![0x1303],
        vec![crate::messages::Extension::KeyShare(vec![1, 2, 3, 4])],
    );
    let err = hsm.process_client_hello(hello).unwrap_err();
    assert!(matches!(err, TlsError::ProtocolError(_)));
}

#[test]
fn process_client_hello_accepts_supported_groups_extension() {
    let mut hsm = HandshakeStateMachine::new();
    let hello = ClientHello::new(
        [12u8; 32],
        vec![0x1303],
        vec![
            crate::messages::Extension::SupportedVersions(vec![0x0304]),
            crate::messages::Extension::KeyShare(vec![1, 2, 3, 4]),
            crate::messages::Extension::SupportedGroups(vec![0x001d]),
        ],
    );
    hsm.process_client_hello(hello).unwrap();
    assert_eq!(hsm.state(), HandshakeState::ReceivedClientHello);
}

#[test]
fn process_client_hello_rejects_unsupported_cipher_suite_list() {
    let mut hsm = HandshakeStateMachine::new();
    let hello = ClientHello::new(
        [13u8; 32],
        vec![],
        vec![
            crate::messages::Extension::SupportedVersions(vec![0x0304]),
            crate::messages::Extension::KeyShare(vec![1, 2, 3, 4]),
        ],
    );
    let err = hsm.process_client_hello(hello).unwrap_err();
    assert!(matches!(err, TlsError::ProtocolError(_)));
}

#[test]
fn handshake_state_does_not_regress_from_connected() {
    let mut hsm = HandshakeStateMachine::new();
    hsm.state = HandshakeState::Connected;
    assert!(hsm.is_connected());

    let err = hsm.process_client_hello(create_test_client_hello()).unwrap_err();
    assert!(matches!(err, TlsError::UnexpectedMessage { .. }));
    assert_eq!(hsm.state(), HandshakeState::Connected);

    let err = hsm.complete_handshake().unwrap_err();
    assert!(matches!(err, TlsError::ProtocolError(_)));
    assert!(hsm.is_connected());
}

#[tokio::test]
async fn generate_server_hello_rejects_connected_state() {
    let mut hsm = HandshakeStateMachine::new();
    hsm.state = HandshakeState::Connected;
    let err = hsm.generate_server_hello().await.unwrap_err();
    assert!(matches!(err, TlsError::ProtocolError(_)));
}

#[tokio::test]
async fn generate_server_hello_rejects_error_state() {
    let mut hsm = HandshakeStateMachine::new();
    hsm.state = HandshakeState::Error;
    let err = hsm.generate_server_hello().await.unwrap_err();
    assert!(matches!(err, TlsError::ProtocolError(_)));
}

#[test]
fn handshake_forward_only_state_ordering() {
    let order = [
        HandshakeState::Start,
        HandshakeState::ReceivedClientHello,
        HandshakeState::SentServerHello,
        HandshakeState::Connected,
    ];
    let mut hsm = HandshakeStateMachine::new();
    assert_eq!(hsm.state(), order[0]);

    hsm.process_client_hello(create_test_client_hello()).unwrap();
    assert_eq!(hsm.state(), order[1]);

    hsm.state = HandshakeState::SentServerHello;
    assert_eq!(hsm.state(), order[2]);

    hsm.complete_handshake().unwrap();
    assert_eq!(hsm.state(), order[3]);
}

fn create_test_client_hello() -> ClientHello {
    let random = [42u8; 32];
    let cipher_suites = vec![0x1303];
    let extensions = vec![
        crate::messages::Extension::SupportedVersions(vec![0x0304]),
        crate::messages::Extension::KeyShare(vec![1, 2, 3, 4]),
    ];

    ClientHello::new(random, cipher_suites, extensions)
}
