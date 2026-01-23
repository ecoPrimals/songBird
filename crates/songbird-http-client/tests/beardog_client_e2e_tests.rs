//! End-to-End Integration Tests for BearDog Client
//! 
//! These tests validate the complete RPC flow with Neural API integration.
//! They are marked #[ignore] by default to avoid requiring a running Neural API.

use songbird_http_client::beardog_client::BearDogClient;

// ====================================================================
// E2E TESTS - Full RPC Flow (marked #[ignore] - require Neural API)
// ====================================================================

#[tokio::test]
#[ignore] // Requires Neural API + BearDog running
async fn test_e2e_tls_derive_application_secrets() {
    let client = BearDogClient::new("/tmp/neural-api-nat0.sock");
    
    // Test vectors (32 bytes each for ChaCha20)
    let pre_master_secret = vec![0u8; 32];
    let client_random = vec![1u8; 32];
    let server_random = vec![2u8; 32];
    let transcript_hash = vec![3u8; 32]; // SHA-256 hash of handshake transcript
    
    let result = client
        .tls_derive_application_secrets(&pre_master_secret, &client_random, &server_random, &transcript_hash)
        .await;
    
    assert!(result.is_ok(), "Failed to derive application secrets: {:?}", result.err());
    
    let secrets = result.unwrap();
    
    // Verify key sizes
    assert_eq!(secrets.client_write_key.len(), 32, "Client write key should be 32 bytes");
    assert_eq!(secrets.server_write_key.len(), 32, "Server write key should be 32 bytes");
    assert_eq!(secrets.client_write_iv.len(), 12, "Client write IV should be 12 bytes");
    assert_eq!(secrets.server_write_iv.len(), 12, "Server write IV should be 12 bytes");
    
    // Verify keys are not all zeros (actual derivation happened)
    assert_ne!(secrets.client_write_key, vec![0u8; 32], "Client key should not be all zeros");
    assert_ne!(secrets.server_write_key, vec![0u8; 32], "Server key should not be all zeros");
}

#[tokio::test]
#[ignore] // Requires Neural API + BearDog running
async fn test_e2e_encrypt_decrypt_roundtrip() {
    let client = BearDogClient::new("/tmp/neural-api-nat0.sock");
    
    // Generate key and nonce
    let key = vec![0x42u8; 32]; // ChaCha20 key
    let nonce = vec![0x01u8; 12]; // ChaCha20 nonce
    let plaintext = b"Hello, Pure Rust HTTPS!";
    let aad = b"additional data";
    
    // Encrypt
    let ciphertext = client.encrypt(&key, &nonce, plaintext, aad).await;
    assert!(ciphertext.is_ok(), "Encryption failed: {:?}", ciphertext.err());
    let ciphertext = ciphertext.unwrap();
    
    // Verify ciphertext is different from plaintext
    assert_ne!(ciphertext, plaintext, "Ciphertext should differ from plaintext");
    
    // Verify ciphertext has tag (16 bytes)
    assert!(ciphertext.len() >= plaintext.len() + 16, "Ciphertext should include 16-byte tag");
    
    // Decrypt
    let decrypted = client.decrypt(&key, &nonce, &ciphertext, aad).await;
    assert!(decrypted.is_ok(), "Decryption failed: {:?}", decrypted.err());
    let decrypted = decrypted.unwrap();
    
    // Verify roundtrip
    assert_eq!(decrypted, plaintext, "Decrypted plaintext should match original");
}

#[tokio::test]
#[ignore] // Requires Neural API + BearDog running
async fn test_e2e_generate_keypair() {
    let client = BearDogClient::new("/tmp/neural-api-nat0.sock");
    
    let result = client.generate_keypair().await;
    assert!(result.is_ok(), "Failed to generate keypair: {:?}", result.err());
    
    let (public_key, private_key) = result.unwrap();
    
    // X25519 keys should be 32 bytes
    assert_eq!(public_key.len(), 32, "Public key should be 32 bytes");
    assert_eq!(private_key.len(), 32, "Private key should be 32 bytes");
    
    // Keys should not be all zeros
    assert_ne!(public_key, vec![0u8; 32], "Public key should not be all zeros");
    assert_ne!(private_key, vec![0u8; 32], "Private key should not be all zeros");
}

#[tokio::test]
#[ignore] // Requires Neural API + BearDog running
async fn test_e2e_ecdh_derive() {
    let client = BearDogClient::new("/tmp/neural-api-nat0.sock");
    
    // Generate two keypairs
    let (_public_a, private_a) = client.generate_keypair().await.unwrap();
    let (public_b, _private_b) = client.generate_keypair().await.unwrap();
    
    // Derive shared secret
    let shared_secret = client.ecdh_derive(&private_a, &public_b).await;
    assert!(shared_secret.is_ok(), "ECDH derivation failed: {:?}", shared_secret.err());
    
    let shared_secret = shared_secret.unwrap();
    
    // X25519 shared secret should be 32 bytes
    assert_eq!(shared_secret.len(), 32, "Shared secret should be 32 bytes");
    
    // Should not be all zeros
    assert_ne!(shared_secret, vec![0u8; 32], "Shared secret should not be all zeros");
}

#[tokio::test]
#[ignore] // Requires Neural API + BearDog running
async fn test_e2e_multiple_sequential_calls() {
    let client = BearDogClient::new("/tmp/neural-api-nat0.sock");
    
    // Make multiple calls to verify request ID increment and no interference
    for i in 0..10 {
        let (public_key, _) = client.generate_keypair().await.unwrap();
        assert_eq!(public_key.len(), 32, "Iteration {} failed", i);
    }
}

#[tokio::test]
#[ignore] // Requires Neural API + BearDog running
async fn test_e2e_concurrent_calls() {
    use tokio::task::JoinSet;
    
    let client = std::sync::Arc::new(BearDogClient::new("/tmp/neural-api-nat0.sock"));
    
    let mut set = JoinSet::new();
    
    // Spawn 10 concurrent calls
    for _ in 0..10 {
        let client_clone = client.clone();
        set.spawn(async move {
            client_clone.generate_keypair().await
        });
    }
    
    // Wait for all to complete
    let mut success_count = 0;
    while let Some(result) = set.join_next().await {
        if result.is_ok() && result.unwrap().is_ok() {
            success_count += 1;
        }
    }
    
    assert_eq!(success_count, 10, "All concurrent calls should succeed");
}

#[tokio::test]
#[ignore] // Requires Neural API + BearDog running
async fn test_e2e_large_plaintext() {
    let client = BearDogClient::new("/tmp/neural-api-nat0.sock");
    
    let key = vec![0x42u8; 32];
    let nonce = vec![0x01u8; 12];
    
    // 1 MB plaintext
    let plaintext = vec![0x55u8; 1024 * 1024];
    let aad = b"large data";
    
    // Encrypt
    let ciphertext = client.encrypt(&key, &nonce, &plaintext, aad).await;
    assert!(ciphertext.is_ok(), "Large encryption failed");
    
    let ciphertext = ciphertext.unwrap();
    
    // Decrypt
    let decrypted = client.decrypt(&key, &nonce, &ciphertext, aad).await;
    assert!(decrypted.is_ok(), "Large decryption failed");
    
    assert_eq!(decrypted.unwrap(), plaintext, "Large roundtrip failed");
}

#[tokio::test]
#[ignore] // Requires Neural API + BearDog running
async fn test_e2e_empty_plaintext() {
    let client = BearDogClient::new("/tmp/neural-api-nat0.sock");
    
    let key = vec![0x42u8; 32];
    let nonce = vec![0x01u8; 12];
    let plaintext = b"";
    let aad = b"empty";
    
    // Encrypt empty plaintext
    let ciphertext = client.encrypt(&key, &nonce, plaintext, aad).await;
    assert!(ciphertext.is_ok(), "Empty encryption failed");
    
    let ciphertext = ciphertext.unwrap();
    
    // Should still have tag (16 bytes)
    assert_eq!(ciphertext.len(), 16, "Empty ciphertext should be just the tag");
    
    // Decrypt
    let decrypted = client.decrypt(&key, &nonce, &ciphertext, aad).await;
    assert!(decrypted.is_ok(), "Empty decryption failed");
    assert_eq!(decrypted.unwrap(), plaintext, "Empty roundtrip failed");
}

#[tokio::test]
#[ignore] // Requires Neural API + BearDog running
async fn test_e2e_decrypt_authentication_failure() {
    let client = BearDogClient::new("/tmp/neural-api-nat0.sock");
    
    let key = vec![0x42u8; 32];
    let nonce = vec![0x01u8; 12];
    let plaintext = b"test data";
    let aad = b"aad";
    
    // Encrypt
    let ciphertext = client.encrypt(&key, &nonce, plaintext, aad).await.unwrap();
    
    // Tamper with ciphertext (flip a bit)
    let mut tampered = ciphertext.clone();
    tampered[0] ^= 0x01;
    
    // Decrypt should fail authentication
    let result = client.decrypt(&key, &nonce, &tampered, aad).await;
    assert!(result.is_err(), "Tampered ciphertext should fail authentication");
}

#[tokio::test]
#[ignore] // Requires Neural API + BearDog running
async fn test_e2e_decrypt_wrong_aad() {
    let client = BearDogClient::new("/tmp/neural-api-nat0.sock");
    
    let key = vec![0x42u8; 32];
    let nonce = vec![0x01u8; 12];
    let plaintext = b"test data";
    let aad = b"correct aad";
    
    // Encrypt with correct AAD
    let ciphertext = client.encrypt(&key, &nonce, plaintext, aad).await.unwrap();
    
    // Try to decrypt with wrong AAD
    let wrong_aad = b"wrong aad";
    let result = client.decrypt(&key, &nonce, &ciphertext, wrong_aad).await;
    assert!(result.is_err(), "Wrong AAD should fail authentication");
}

// ====================================================================
// CHAOS E2E TESTS - Extreme Conditions
// ====================================================================

#[tokio::test]
#[ignore]
async fn test_chaos_e2e_rapid_fire_requests() {
    let client = BearDogClient::new("/tmp/neural-api-nat0.sock");
    
    // Make 100 requests as fast as possible
    for _ in 0..100 {
        let _ = client.generate_keypair().await;
        // Don't check result, just verify no panic
    }
}

#[tokio::test]
#[ignore]
async fn test_chaos_e2e_alternating_operations() {
    let client = BearDogClient::new("/tmp/neural-api-nat0.sock");
    
    let key = vec![0x42u8; 32];
    let nonce = vec![0x01u8; 12];
    
    for i in 0..20 {
        if i % 3 == 0 {
            let _ = client.generate_keypair().await;
        } else if i % 3 == 1 {
            let _ = client.encrypt(&key, &nonce, b"test", b"aad").await;
        } else {
            let pre_master = vec![0u8; 32];
            let client_random = vec![1u8; 32];
            let server_random = vec![2u8; 32];
            let transcript_hash = vec![3u8; 32];
            let cipher_suite = 0x1303; // ChaCha20-Poly1305
            let _ = client.tls_derive_application_secrets(&pre_master, &client_random, &server_random, &transcript_hash, cipher_suite).await;
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_chaos_e2e_varying_sizes() {
    let client = BearDogClient::new("/tmp/neural-api-nat0.sock");
    
    let key = vec![0x42u8; 32];
    let nonce = vec![0x01u8; 12];
    
    // Test various plaintext sizes
    let sizes = vec![0, 1, 15, 16, 17, 100, 1024, 4096, 65536];
    
    for size in sizes {
        let plaintext = vec![0x55u8; size];
        let result = client.encrypt(&key, &nonce, &plaintext, b"aad").await;
        assert!(result.is_ok(), "Failed for size {}", size);
    }
}

// ====================================================================
// FAULT E2E TESTS - Error Conditions
// ====================================================================

#[tokio::test]
async fn test_fault_e2e_invalid_socket_path() {
    let client = BearDogClient::new("/nonexistent/socket.sock");
    
    let result = client.generate_keypair().await;
    assert!(result.is_err(), "Should fail with invalid socket path");
}

#[tokio::test]
async fn test_fault_e2e_empty_socket_path() {
    let client = BearDogClient::new("");
    
    let result = client.generate_keypair().await;
    assert!(result.is_err(), "Should fail with empty socket path");
}

#[tokio::test]
#[ignore] // Requires Neural API running
async fn test_fault_e2e_short_ciphertext() {
    let client = BearDogClient::new("/tmp/neural-api-nat0.sock");
    
    let key = vec![0x42u8; 32];
    let nonce = vec![0x01u8; 12];
    
    // Ciphertext too short (need at least 16 bytes for tag)
    let short_ciphertext = vec![0u8; 10];
    
    let result = client.decrypt(&key, &nonce, &short_ciphertext, b"aad").await;
    assert!(result.is_err(), "Should fail with ciphertext < 16 bytes");
}

#[tokio::test]
#[ignore] // Requires Neural API running
async fn test_fault_e2e_wrong_key_size() {
    let client = BearDogClient::new("/tmp/neural-api-nat0.sock");
    
    // Wrong key size (ChaCha20 needs 32 bytes)
    let wrong_key = vec![0x42u8; 16];
    let nonce = vec![0x01u8; 12];
    
    let result = client.encrypt(&wrong_key, &nonce, b"test", b"aad").await;
    // BearDog should reject this
    assert!(result.is_err(), "Should fail with wrong key size");
}

#[tokio::test]
#[ignore] // Requires Neural API running
async fn test_fault_e2e_wrong_nonce_size() {
    let client = BearDogClient::new("/tmp/neural-api-nat0.sock");
    
    let key = vec![0x42u8; 32];
    // Wrong nonce size (ChaCha20 needs 12 bytes)
    let wrong_nonce = vec![0x01u8; 24];
    
    let result = client.encrypt(&key, &wrong_nonce, b"test", b"aad").await;
    // BearDog should reject this
    assert!(result.is_err(), "Should fail with wrong nonce size");
}

#[tokio::test]
#[ignore] // Requires Neural API running
async fn test_fault_e2e_wrong_secret_size() {
    let client = BearDogClient::new("/tmp/neural-api-nat0.sock");
    
    // Wrong sizes for TLS derivation (need 32 bytes each)
    let short_secret = vec![0u8; 16];
    let client_random = vec![1u8; 32];
    let server_random = vec![2u8; 32];
    let transcript_hash = vec![3u8; 32];
    let cipher_suite = 0x1303; // ChaCha20-Poly1305
    
    let result = client.tls_derive_application_secrets(&short_secret, &client_random, &server_random, &transcript_hash, cipher_suite).await;
    // BearDog should reject this
    assert!(result.is_err(), "Should fail with wrong secret size");
}

#[tokio::test]
#[ignore] // Requires Neural API running
async fn test_fault_e2e_invalid_base64_response() {
    // This would only happen if BearDog or Neural API is buggy
    // We can't easily inject this in a real test, but it's here for documentation
    // In practice, if BearDog returns invalid base64, our decode will fail gracefully
}

