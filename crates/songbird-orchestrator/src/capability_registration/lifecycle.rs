// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Register, unregister, and probe Neural API availability.

use anyhow::Result;
use songbird_types::primal_names;
use std::env;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{info, warn};

use super::config::CapabilityRegistrationConfig;
use super::payload::{capability_registration_params, capability_unregister_params};
use super::transport::connect_platform;

/// Register Songbird's capabilities with the Neural API
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn register_capabilities() -> Result<()> {
    let config = CapabilityRegistrationConfig::from_env()?;
    register_capabilities_with(&config).await
}

/// Register capabilities with explicit config (concurrent-safe, testable)
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn register_capabilities_with(config: &CapabilityRegistrationConfig) -> Result<()> {
    info!("🔄 Registering capabilities with Neural API...");

    let neural_socket = &config.neural_socket;
    let songbird_socket = &config.songbird_socket;
    let primal_id = &config.primal_id;

    let family_id = env::var("FAMILY_ID")
        .or_else(|_| env::var("SONGBIRD_FAMILY_ID"))
        .unwrap_or_else(|_| "default".to_string());

    let registration = capability_registration_params(
        primal_id,
        songbird_socket,
        &family_id,
        env!("CARGO_PKG_VERSION"),
    );

    let mut stream = match connect_platform(neural_socket).await {
        Ok(s) => s,
        Err(e) => {
            warn!("⚠️  Failed to connect to Neural API at {}: {}", neural_socket, e);
            warn!("   Songbird will continue without Neural API registration");
            warn!("   Direct socket connections will still work");
            return Ok(());
        }
    };

    let request = format!("{registration}\n");
    if let Err(e) = stream.write_all(request.as_bytes()).await {
        warn!("⚠️  Failed to send registration to Neural API: {}", e);
        return Ok(());
    }

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    if let Err(e) = reader.read_line(&mut response).await {
        warn!("⚠️  Failed to read registration response: {}", e);
        return Ok(());
    }

    let response_json: serde_json::Value = match serde_json::from_str(&response) {
        Ok(j) => j,
        Err(e) => {
            warn!("⚠️  Failed to parse registration response: {}", e);
            warn!("   Response: {}", response);
            return Ok(());
        }
    };

    if response_json.get("result").is_some() {
        info!("✅ Capabilities registered successfully with Neural API");
        info!("   Capability: secure_http");
        info!(
            "   Operations: http.get, http.post, http.put, http.delete, http.patch, http.request"
        );
        info!("   Primal ID: {}", primal_id);
        info!("   Socket: {}", songbird_socket);
        info!("   Neural API: {}", neural_socket);
    } else if let Some(error) = response_json.get("error") {
        warn!("⚠️  Neural API registration returned error: {:?}", error);
        warn!("   Songbird will continue without registration");
        warn!("   Direct socket connections will still work");
    } else {
        warn!("⚠️  Unexpected registration response from Neural API");
        warn!("   Response: {}", response);
    }

    Ok(())
}

/// Unregister capabilities on shutdown (optional but recommended)
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn unregister_capabilities() -> Result<()> {
    let neural_socket = env::var("NEURAL_API_SOCKET").unwrap_or_else(|_| {
        if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
            format!("{runtime_dir}/biomeos/neural-api.sock")
        } else {
            "/tmp/biomeos/neural-api.sock".to_string()
        }
    });
    let primal_id = env::var("PRIMAL_ID")
        .or_else(|_| env::var("SONGBIRD_PRIMAL_ID"))
        .unwrap_or_else(|_| primal_names::SELF_NAME.to_string());
    unregister_capabilities_with(&neural_socket, &primal_id).await
}

/// Unregister capabilities with explicit config (concurrent-safe, testable)
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn unregister_capabilities_with(neural_socket: &str, primal_id: &str) -> Result<()> {
    info!("🔄 Unregistering capabilities from Neural API...");

    let unregister = capability_unregister_params(primal_id);

    match connect_platform(neural_socket).await {
        Ok(mut stream) => {
            let request = format!("{unregister}\n");
            match stream.write_all(request.as_bytes()).await {
                Ok(()) => {
                    info!("✅ Capabilities unregistered from Neural API");
                    info!("   Primal ID: {}", primal_id);
                }
                Err(e) => {
                    warn!("⚠️  Failed to send unregister request: {}", e);
                    warn!("   This is OK during shutdown");
                }
            }
        }
        Err(_) => {
            info!("   Neural API not available for unregistration (this is OK)");
        }
    }

    Ok(())
}

/// Check if Neural API is available
pub async fn check_neural_api_available() -> bool {
    let neural_socket = env::var("NEURAL_API_SOCKET").unwrap_or_else(|_| {
        if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
            format!("{runtime_dir}/biomeos/neural-api.sock")
        } else {
            "/tmp/biomeos/neural-api.sock".to_string()
        }
    });
    check_neural_api_available_at(&neural_socket).await
}

/// Check Neural API at explicit socket path (concurrent-safe, testable)
pub async fn check_neural_api_available_at(neural_socket: &str) -> bool {
    match connect_platform(neural_socket).await {
        Ok(_) => {
            info!("✅ Neural API available at {}", neural_socket);
            true
        }
        Err(e) => {
            warn!("⚠️  Neural API not available at {}: {}", neural_socket, e);
            false
        }
    }
}
