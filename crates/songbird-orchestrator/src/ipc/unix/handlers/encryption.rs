// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2025 ecoPrimals
//! Encryption Wrapper Handlers
//!
//! Handlers for encrypting/decrypting discovery broadcasts via BearDog.
//! These methods delegate cryptographic operations to the BearDog primal.

use serde_json::Value;
use tracing::info;

use crate::ipc::jsonrpc::JsonRpcError;

/// Handle encrypt_discovery - Encrypt payload for discovery broadcast
///
/// NEW (Feb 4, 2026): biomeOS integration for beacon exchange.
/// Delegates to BearDog's `beacon.encrypt` method.
pub async fn handle_encrypt_discovery(params: Option<Value>) -> Result<Value, JsonRpcError> {
    use base64::{Engine as _, engine::general_purpose};
    
    #[derive(serde::Deserialize)]
    struct EncryptParams {
        payload: Value,
        #[serde(default)]
        use_beacon_seed: bool,
    }
    
    let params: EncryptParams = match params {
        Some(p) => serde_json::from_value(p)
            .map_err(|e| JsonRpcError::invalid_params(e.to_string()))?,
        None => return Err(JsonRpcError::invalid_params("Missing params")),
    };
    
    info!("🔐 Encrypting discovery payload for broadcast");
    
    // Serialize payload to JSON bytes
    let payload_json = serde_json::to_vec(&params.payload)
        .map_err(|e| JsonRpcError::internal_error(&format!("Failed to serialize payload: {}", e)))?;
    
    // Base64 encode for BearDog
    let payload_b64 = general_purpose::STANDARD.encode(&payload_json);
    
    // Call BearDog's beacon.encrypt method
    let beardog_socket = songbird_http_client::discover_beardog_socket();
    let encrypted_b64 = call_beardog_method(
        &beardog_socket,
        "beacon.encrypt",
        serde_json::json!({"plaintext_b64": payload_b64})
    )
    .await
    .map_err(|e| JsonRpcError::internal_error(&format!("BearDog encryption failed: {}", e)))?;
    
    let ciphertext_b64 = encrypted_b64["ciphertext_b64"]
        .as_str()
        .ok_or_else(|| JsonRpcError::internal_error("Missing ciphertext_b64 in BearDog response"))?
        .to_string();
    
    info!("✅ Payload encrypted successfully");
    
    Ok(serde_json::json!({
        "encrypted_b64": ciphertext_b64
    }))
}

/// Handle decrypt_discovery - Decrypt discovery broadcast
///
/// NEW (Feb 4, 2026): biomeOS integration for beacon exchange.
/// Delegates to BearDog's `beacon.try_decrypt` method.
pub async fn handle_decrypt_discovery(params: Option<Value>) -> Result<Value, JsonRpcError> {
    use base64::{Engine as _, engine::general_purpose};
    
    #[derive(serde::Deserialize)]
    struct DecryptParams {
        encrypted_b64: String,
        known_beacon_seeds: Vec<String>,
    }
    
    let params: DecryptParams = match params {
        Some(p) => serde_json::from_value(p)
            .map_err(|e| JsonRpcError::invalid_params(e.to_string()))?,
        None => return Err(JsonRpcError::invalid_params("Missing params")),
    };
    
    info!("🔓 Attempting to decrypt discovery payload ({} known seeds)", params.known_beacon_seeds.len());
    
    let beardog_socket = songbird_http_client::discover_beardog_socket();
    
    // Try each known beacon seed
    for (index, seed_hex) in params.known_beacon_seeds.iter().enumerate() {
        let result = call_beardog_method(
            &beardog_socket,
            "beacon.try_decrypt",
            serde_json::json!({
                "ciphertext_b64": params.encrypted_b64,
                "seed_hex": seed_hex
            })
        )
        .await;
        
        match result {
            Ok(response) if response["decrypted"].as_bool().unwrap_or(false) => {
                // Decode base64
                let plaintext_b64 = response["plaintext_b64"]
                    .as_str()
                    .ok_or_else(|| JsonRpcError::internal_error("Missing plaintext_b64"))?;
                
                let plaintext_bytes = general_purpose::STANDARD.decode(plaintext_b64)
                    .map_err(|e| JsonRpcError::internal_error(&format!("Failed to decode plaintext: {}", e)))?;
                
                // Parse JSON
                let payload: Value = serde_json::from_slice(&plaintext_bytes)
                    .map_err(|e| JsonRpcError::internal_error(&format!("Failed to parse decrypted payload: {}", e)))?;
                
                info!("✅ Decryption successful with seed #{}", index);
                
                return Ok(serde_json::json!({
                    "decrypted": true,
                    "payload": payload,
                    "matched_seed_index": index
                }));
            }
            _ => continue,
        }
    }
    
    info!("❌ Decryption failed - no matching seed found");
    
    Ok(serde_json::json!({
        "decrypted": false,
        "payload": Value::Null,
        "matched_seed_index": Value::Null
    }))
}

/// Call BearDog method via Unix socket JSON-RPC
async fn call_beardog_method(
    socket_path: &str,
    method: &str,
    params: Value,
) -> anyhow::Result<Value> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;
    
    let stream = UnixStream::connect(socket_path).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    
    let request_str = serde_json::to_string(&request)?;
    writer.write_all(request_str.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    
    let mut response_line = String::new();
    reader.read_line(&mut response_line).await?;
    
    let response: Value = serde_json::from_str(&response_line)?;
    
    if let Some(error) = response.get("error") {
        return Err(anyhow::anyhow!("BearDog error: {}", error));
    }
    
    Ok(response["result"].clone())
}
