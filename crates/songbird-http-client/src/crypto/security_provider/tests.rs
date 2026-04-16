// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::{RoutingMode, SecurityCryptoProvider};
use crate::crypto::capability::CryptoCapability;

fn test_security_socket_path() -> String {
    tempfile::env::temp_dir().join("songbird-test-security.sock").to_string_lossy().into_owned()
}

#[test]
fn test_provider_creation() {
    let path = test_security_socket_path();
    let provider = SecurityCryptoProvider::new(path.clone());
    assert_eq!(provider.name(), "security provider");
    assert_eq!(provider.socket_path(), path);
    assert_eq!(provider.mode, RoutingMode::Direct);
}

#[test]
fn test_semantic_mapping() {
    let _provider = SecurityCryptoProvider::new(test_security_socket_path());

    assert_eq!(
        SecurityCryptoProvider::semantic_to_actual("crypto.generate_keypair"),
        "crypto.x25519_generate_ephemeral"
    );
    assert_eq!(
        SecurityCryptoProvider::semantic_to_actual("crypto.ecdh_derive"),
        "crypto.x25519_derive_secret"
    );
}

#[test]
fn test_capability_mapping() {
    let _provider = SecurityCryptoProvider::new(test_security_socket_path());

    assert_eq!(
        SecurityCryptoProvider::method_to_capability("crypto.generate_keypair"),
        ("crypto", "generate_keypair")
    );
    assert_eq!(
        SecurityCryptoProvider::method_to_capability("crypto.ecdh_derive"),
        ("crypto", "derive_secret")
    );
    assert_eq!(SecurityCryptoProvider::method_to_capability("crypto.sha256"), ("crypto", "sha256"));
    assert_eq!(
        SecurityCryptoProvider::method_to_capability("tls.derive_handshake_secrets"),
        ("tls_crypto", "derive_handshake_secrets")
    );
}

#[test]
fn test_neural_api_mode() {
    let provider =
        SecurityCryptoProvider::with_mode("/tmp/neural-api.sock", RoutingMode::NeuralApi);
    assert_eq!(provider.mode, RoutingMode::NeuralApi);
    assert_eq!(provider.socket_path(), "/tmp/neural-api.sock");
}

#[test]
fn test_direct_mode() {
    let path = test_security_socket_path();
    let provider = SecurityCryptoProvider::new(path.clone());
    assert_eq!(provider.mode, RoutingMode::Direct);
    assert_eq!(provider.socket_path(), path);
}
