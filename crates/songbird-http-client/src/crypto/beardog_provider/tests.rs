// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use super::{BearDogProvider, RoutingMode};
use crate::crypto::CryptoCapability;

#[test]
fn test_provider_creation() {
    let provider = BearDogProvider::new("/tmp/beardog.sock");
    assert_eq!(provider.name(), "BearDog");
    assert_eq!(provider.socket_path(), "/tmp/beardog.sock");
    assert_eq!(provider.mode, RoutingMode::Direct);
}

#[test]
fn test_semantic_mapping() {
    let _provider = BearDogProvider::new("/tmp/beardog.sock");

    assert_eq!(
        BearDogProvider::semantic_to_actual("crypto.generate_keypair"),
        "crypto.x25519_generate_ephemeral"
    );
    assert_eq!(
        BearDogProvider::semantic_to_actual("crypto.ecdh_derive"),
        "crypto.x25519_derive_secret"
    );
}

#[test]
fn test_capability_mapping() {
    let _provider = BearDogProvider::new("/tmp/beardog.sock");

    assert_eq!(
        BearDogProvider::method_to_capability("crypto.generate_keypair"),
        ("crypto", "generate_keypair")
    );
    assert_eq!(
        BearDogProvider::method_to_capability("crypto.ecdh_derive"),
        ("crypto", "derive_secret")
    );
    assert_eq!(BearDogProvider::method_to_capability("crypto.sha256"), ("crypto", "sha256"));
    assert_eq!(
        BearDogProvider::method_to_capability("tls.derive_handshake_secrets"),
        ("tls_crypto", "derive_handshake_secrets")
    );
}

#[test]
fn test_neural_api_mode() {
    let provider = BearDogProvider::with_mode("/tmp/neural-api.sock", RoutingMode::NeuralApi);
    assert_eq!(provider.mode, RoutingMode::NeuralApi);
    assert_eq!(provider.socket_path(), "/tmp/neural-api.sock");
}

#[test]
fn test_direct_mode() {
    let provider = BearDogProvider::new("/tmp/beardog.sock");
    assert_eq!(provider.mode, RoutingMode::Direct);
    assert_eq!(provider.socket_path(), "/tmp/beardog.sock");
}
