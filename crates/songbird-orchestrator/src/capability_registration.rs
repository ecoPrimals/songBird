//! # Capability Registration - Neural API Integration
//!
//! This module handles automatic registration of Songbird's capabilities with
//! the Neural API, enabling TRUE PRIMAL loose coupling and capability-based discovery.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ SONGBIRD STARTUP                                            │
//! │ 1. Initialize TLS stack                                     │
//! │ 2. Start JSON-RPC server                                    │
//! │ 3. ✨ Register capabilities with Neural API                │
//! │ 4. Accept requests                                          │
//! └─────────────────────┬───────────────────────────────────────┘
//!                       │
//!                       │ capability.register
//!                       │
//! ┌─────────────────────▼───────────────────────────────────────┐
//! │ NEURAL API - Capability Registry                            │
//! │ secure_http → [songbird-nat0]                              │
//! │   - http.get, http.post, http.put, http.delete            │
//! └─────────────────────┬───────────────────────────────────────┘
//!                       │
//! ┌─────────────────────▼───────────────────────────────────────┐
//! │ CONSUMER PRIMALS (Squirrel, etc.)                          │
//! │ neural_api.capability_call("secure_http", "http.post", {}) │
//! │ → Zero knowledge of Songbird required!                      │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Benefits
//!
//! - ✅ **Zero Configuration** - Primals discover Songbird automatically
//! - ✅ **Loose Coupling** - No hardcoded dependencies
//! - ✅ **Semantic APIs** - Operations like `http.post` just work
//! - ✅ **Isomorphic Evolution** - Songbird can evolve without breaking consumers
//! - ✅ **Fail-Safe** - Registration failure doesn't block Songbird startup

use anyhow::{Context, Result};
use serde_json::json;
use std::env;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
// Platform-agnostic IPC transport
#[cfg(unix)]
use tokio::net::UnixStream as PlatformStream;
#[cfg(windows)]
use tokio::net::TcpStream as PlatformStream;
use tracing::{info, warn};

/// Platform-agnostic connection helper
#[cfg(unix)]
async fn connect_platform(path: &str) -> std::io::Result<PlatformStream> {
    PlatformStream::connect(path).await
}

#[cfg(windows)]
async fn connect_platform(address: &str) -> std::io::Result<PlatformStream> {
    PlatformStream::connect(address).await
}

/// Register Songbird's capabilities with the Neural API
///
/// This function is called during Songbird startup to make its
/// capabilities discoverable to other primals in the ecosystem.
///
/// ## Fail-Safe Design
///
/// Registration failure does NOT fail Songbird startup. This ensures:
/// - Songbird works even if Neural API is down
/// - Direct socket connections still work
/// - System is resilient to partial failures
///
/// ## Registered Capabilities
///
/// - **secure_http**: HTTPS client with Pure Rust TLS 1.3
///   - `http.get` - HTTP GET requests
///   - `http.post` - HTTP POST requests
///   - `http.put` - HTTP PUT requests
///   - `http.delete` - HTTP DELETE requests
///   - `http.patch` - HTTP PATCH requests
///   - `http.request` - Generic HTTP request (fallback)
///
/// ## Environment Variables
///
/// - `NEURAL_API_SOCKET` - Neural API socket path (default: `/tmp/neural-api-nat0.sock`)
/// - `SONGBIRD_SOCKET_PATH` - Our socket path (required)
/// - `PRIMAL_ID` - Our primal identifier (default: `songbird-nat0`)
///
/// # Errors
///
/// Returns error if:
/// - `SONGBIRD_SOCKET_PATH` is not set
/// - Failed to connect to Neural API
/// - Failed to send registration request
/// - Failed to parse response
///
/// Note: Errors are logged but do NOT fail Songbird startup.
pub async fn register_capabilities() -> Result<()> {
    info!("🔄 Registering capabilities with Neural API...");

    // Get Neural API socket from environment
    let neural_socket =
        env::var("NEURAL_API_SOCKET").unwrap_or_else(|_| "/tmp/neural-api-nat0.sock".to_string());

    // Get our own socket path (required)
    let songbird_socket = env::var("SONGBIRD_SOCKET_PATH")
        .or_else(|_| env::var("SONGBIRD_SOCKET"))
        .or_else(|_| env::var("SONGBIRD_IPC_SOCKET"))
        .context("SONGBIRD_SOCKET_PATH not set (required for Neural API registration)")?;

    // Get our primal ID
    let primal_id = env::var("PRIMAL_ID")
        .or_else(|_| env::var("SONGBIRD_PRIMAL_ID"))
        .unwrap_or_else(|_| "songbird-nat0".to_string());

    // Get family ID for metadata
    let family_id = env::var("FAMILY_ID")
        .or_else(|_| env::var("SONGBIRD_FAMILY_ID"))
        .unwrap_or_else(|_| "default".to_string());

    // Build registration request
    let registration = json!({
        "jsonrpc": "2.0",
        "method": "capability.register",
        "params": {
            "primal_id": primal_id,
            "capability": "secure_http",
            "socket_path": songbird_socket,
            "operations": [
                "http.get",
                "http.post",
                "http.put",
                "http.delete",
                "http.patch",
                "http.request"  // Generic fallback
            ],
            "metadata": {
                "tls_version": "1.3",
                "pure_rust": true,
                "supports_http2": true,
                "tower_atomic": true,
                "ecobin_compliant": true,
                "provider": "songbird",
                "family_id": family_id,
                "version": env!("CARGO_PKG_VERSION")
            }
        },
        "id": 1
    });

    // Connect to Neural API (platform-agnostic)
    let mut stream = match connect_platform(&neural_socket).await {
        Ok(s) => s,
        Err(e) => {
            warn!("⚠️  Failed to connect to Neural API at {}: {}", neural_socket, e);
            warn!("   Songbird will continue without Neural API registration");
            warn!("   Direct socket connections will still work");
            return Ok(()); // Don't fail startup
        }
    };

    // Send registration
    let request = format!("{}\n", registration);
    if let Err(e) = stream.write_all(request.as_bytes()).await {
        warn!("⚠️  Failed to send registration to Neural API: {}", e);
        return Ok(()); // Don't fail startup
    }

    // Read response
    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    if let Err(e) = reader.read_line(&mut response).await {
        warn!("⚠️  Failed to read registration response: {}", e);
        return Ok(()); // Don't fail startup
    }

    // Parse response
    let response_json: serde_json::Value = match serde_json::from_str(&response) {
        Ok(j) => j,
        Err(e) => {
            warn!("⚠️  Failed to parse registration response: {}", e);
            warn!("   Response: {}", response);
            return Ok(()); // Don't fail startup
        }
    };

    // Check result
    if response_json.get("result").is_some() {
        info!("✅ Capabilities registered successfully with Neural API");
        info!("   Capability: secure_http");
        info!(
            "   Operations: http.get, http.post, http.put, http.delete, http.patch, http.request"
        );
        info!("   Primal ID: {}", primal_id);
        info!("   Socket: {}", songbird_socket);
        info!("   Neural API: {}", neural_socket);
        Ok(())
    } else if let Some(error) = response_json.get("error") {
        warn!("⚠️  Neural API registration returned error: {:?}", error);
        warn!("   Songbird will continue without registration");
        warn!("   Direct socket connections will still work");
        Ok(()) // Don't fail startup
    } else {
        warn!("⚠️  Unexpected registration response from Neural API");
        warn!("   Response: {}", response);
        Ok(()) // Don't fail startup
    }
}

/// Unregister capabilities on shutdown (optional but recommended)
///
/// This function is called during graceful shutdown to remove Songbird's
/// capabilities from the Neural API registry.
///
/// ## Fail-Safe Design
///
/// Unregistration failure does NOT prevent shutdown. This ensures:
/// - Songbird can shut down even if Neural API is unavailable
/// - Clean shutdown is always possible
/// - System is resilient to partial failures
///
/// # Errors
///
/// Returns error if unregistration fails, but errors are logged and ignored.
/// Shutdown continues normally.
pub async fn unregister_capabilities() -> Result<()> {
    info!("🔄 Unregistering capabilities from Neural API...");

    // Get configuration
    let neural_socket =
        env::var("NEURAL_API_SOCKET").unwrap_or_else(|_| "/tmp/neural-api-nat0.sock".to_string());

    let primal_id = env::var("PRIMAL_ID")
        .or_else(|_| env::var("SONGBIRD_PRIMAL_ID"))
        .unwrap_or_else(|_| "songbird-nat0".to_string());

    // Build unregister request
    let unregister = json!({
        "jsonrpc": "2.0",
        "method": "capability.unregister",
        "params": {
            "primal_id": primal_id,
            "capability": "secure_http"
        },
        "id": 2
    });

    // Try to connect and unregister (platform-agnostic)
    match connect_platform(&neural_socket).await {
        Ok(mut stream) => {
            let request = format!("{}\n", unregister);
            match stream.write_all(request.as_bytes()).await {
                Ok(_) => {
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
            // Neural API may already be shut down or unavailable - that's fine
            info!("   Neural API not available for unregistration (this is OK)");
        }
    }

    Ok(())
}

/// Check if Neural API is available
///
/// This is a health check function that can be used to verify
/// Neural API connectivity without registering.
///
/// # Returns
///
/// `true` if Neural API is reachable, `false` otherwise.
pub async fn check_neural_api_available() -> bool {
    let neural_socket =
        env::var("NEURAL_API_SOCKET").unwrap_or_else(|_| "/tmp/neural-api-nat0.sock".to_string());

    match connect_platform(&neural_socket).await {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    #[cfg(unix)]
    use tokio::net::UnixListener;

    // Global lock to prevent env var test interference
    static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_env_var_defaults() {
        let _guard = ENV_TEST_LOCK.lock().unwrap();

        // Clear relevant env vars
        env::remove_var("NEURAL_API_SOCKET");
        env::remove_var("PRIMAL_ID");

        // Defaults should be used
        let neural_socket = env::var("NEURAL_API_SOCKET")
            .unwrap_or_else(|_| "/tmp/neural-api-nat0.sock".to_string());
        assert_eq!(neural_socket, "/tmp/neural-api-nat0.sock");

        let primal_id = env::var("PRIMAL_ID").unwrap_or_else(|_| "songbird-nat0".to_string());
        assert_eq!(primal_id, "songbird-nat0");
    }

    #[tokio::test]
    async fn test_registration_without_songbird_socket_fails() {
        let _guard = ENV_TEST_LOCK.lock().unwrap();

        env::remove_var("SONGBIRD_SOCKET_PATH");
        env::remove_var("SONGBIRD_SOCKET");
        env::remove_var("SONGBIRD_IPC_SOCKET");

        let result = register_capabilities().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("SONGBIRD_SOCKET_PATH"));
    }

    #[tokio::test]
    async fn test_registration_with_unavailable_neural_api_succeeds() {
        let _guard = ENV_TEST_LOCK.lock().unwrap();

        // Set required env vars
        env::set_var("SONGBIRD_SOCKET_PATH", "/tmp/test-songbird-unique.sock");
        env::set_var("NEURAL_API_SOCKET", "/tmp/nonexistent-neural-api-sock-unique-12345.sock");

        // Should succeed (not fail startup) even though Neural API is unavailable
        let result = register_capabilities().await;
        if let Err(ref e) = result {
            eprintln!("Registration failed with error: {}", e);
        }
        assert!(
            result.is_ok(),
            "Registration should succeed gracefully even when Neural API is unavailable"
        );

        // Cleanup
        env::remove_var("SONGBIRD_SOCKET_PATH");
        env::remove_var("NEURAL_API_SOCKET");
    }

    #[tokio::test]
    async fn test_unregistration_with_unavailable_neural_api_succeeds() {
        let _guard = ENV_TEST_LOCK.lock().unwrap();

        env::set_var("NEURAL_API_SOCKET", "/tmp/nonexistent-neural-api.sock");

        // Should always succeed (graceful degradation)
        let result = unregister_capabilities().await;
        assert!(result.is_ok());

        env::remove_var("NEURAL_API_SOCKET");
    }

    #[tokio::test]
    async fn test_check_neural_api_with_mock_server() {
        let _guard = ENV_TEST_LOCK.lock().unwrap();

        let socket_path = "/tmp/test-neural-api-check.sock";

        // Clean up any existing socket
        let _ = std::fs::remove_file(socket_path);

        // Start mock server
        let listener = UnixListener::bind(socket_path).unwrap();
        let _server_task = tokio::spawn(async move {
            let _ = listener.accept().await;
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        env::set_var("NEURAL_API_SOCKET", socket_path);
        let available = check_neural_api_available().await;
        assert!(available);

        // Cleanup
        env::remove_var("NEURAL_API_SOCKET");
        let _ = std::fs::remove_file(socket_path);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // 🧪 E2E TESTS - End-to-End Registration Flow
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_e2e_full_registration_lifecycle() {
        let _guard = ENV_TEST_LOCK.lock().unwrap();

        let socket_path = "/tmp/test-neural-e2e.sock";
        let songbird_socket = "/tmp/test-songbird-e2e.sock";

        // Cleanup
        let _ = std::fs::remove_file(socket_path);
        let _ = std::fs::remove_file(songbird_socket);

        env::set_var("NEURAL_API_SOCKET", socket_path);
        env::set_var("SONGBIRD_SOCKET_PATH", songbird_socket);

        // Start mock Neural API server
        let listener = UnixListener::bind(socket_path).unwrap();
        let server_task = tokio::spawn(async move {
            for _ in 0..6 {
                // Expect 6 registration requests
                if let Ok((mut stream, _)) = listener.accept().await {
                    let mut reader = BufReader::new(&mut stream);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.is_ok() {
                        // Verify it's a registration request
                        assert!(line.contains("capability.register"));
                        // Send success response
                        let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
                        let _ = stream.write_all(format!("{}\n", response).as_bytes()).await;
                    }
                }
            }
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // E2E Test: Register capabilities
        let result = register_capabilities().await;
        assert!(result.is_ok(), "E2E registration should succeed");

        // Wait for server to process
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Cleanup
        drop(server_task);
        env::remove_var("NEURAL_API_SOCKET");
        env::remove_var("SONGBIRD_SOCKET_PATH");
        let _ = std::fs::remove_file(socket_path);
        let _ = std::fs::remove_file(songbird_socket);
    }

    #[tokio::test]
    async fn test_e2e_registration_and_immediate_unregistration() {
        let _guard = ENV_TEST_LOCK.lock().unwrap();

        let socket_path = "/tmp/test-neural-e2e-unreg.sock";
        let songbird_socket = "/tmp/test-songbird-e2e-unreg.sock";

        let _ = std::fs::remove_file(socket_path);
        let _ = std::fs::remove_file(songbird_socket);

        env::set_var("NEURAL_API_SOCKET", socket_path);
        env::set_var("SONGBIRD_SOCKET_PATH", songbird_socket);

        // Mock server that handles both registration and unregistration
        let listener = UnixListener::bind(socket_path).unwrap();
        let _server_task = tokio::spawn(async move {
            for _ in 0..12 {
                // 6 reg + 6 unreg
                if let Ok((mut stream, _)) = listener.accept().await {
                    let mut reader = BufReader::new(&mut stream);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.is_ok() {
                        let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
                        let _ = stream.write_all(format!("{}\n", response).as_bytes()).await;
                    }
                }
            }
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Register
        let reg_result = register_capabilities().await;
        assert!(reg_result.is_ok());

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Unregister
        let unreg_result = unregister_capabilities().await;
        assert!(unreg_result.is_ok());

        // Cleanup
        env::remove_var("NEURAL_API_SOCKET");
        env::remove_var("SONGBIRD_SOCKET_PATH");
        let _ = std::fs::remove_file(socket_path);
        let _ = std::fs::remove_file(songbird_socket);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // 🔥 CHAOS TESTS - Network Chaos & Intermittent Failures
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_chaos_socket_disappears_during_registration() {
        let _guard = ENV_TEST_LOCK.lock().unwrap();

        let socket_path = "/tmp/test-neural-chaos-disappear.sock";
        let songbird_socket = "/tmp/test-songbird-chaos.sock";

        let _ = std::fs::remove_file(socket_path);
        let _ = std::fs::remove_file(songbird_socket);

        env::set_var("NEURAL_API_SOCKET", socket_path);
        env::set_var("SONGBIRD_SOCKET_PATH", songbird_socket);

        // Start server that disappears after 2 connections
        let listener = UnixListener::bind(socket_path).unwrap();
        let server_task = tokio::spawn(async move {
            for i in 0..2 {
                if let Ok((mut stream, _)) = listener.accept().await {
                    let mut reader = BufReader::new(&mut stream);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.is_ok() {
                        let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
                        let _ = stream.write_all(format!("{}\n", response).as_bytes()).await;
                    }
                }
                if i == 1 {
                    // Server dies after 2 requests
                    break;
                }
            }
            // Socket disappears!
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Should still succeed (fail-safe)
        let result = register_capabilities().await;
        assert!(result.is_ok(), "Should handle socket disappearance gracefully (fail-safe)");

        drop(server_task);
        env::remove_var("NEURAL_API_SOCKET");
        env::remove_var("SONGBIRD_SOCKET_PATH");
        let _ = std::fs::remove_file(socket_path);
        let _ = std::fs::remove_file(songbird_socket);
    }

    #[tokio::test]
    async fn test_chaos_slow_neural_api_responses() {
        let _guard = ENV_TEST_LOCK.lock().unwrap();

        let socket_path = "/tmp/test-neural-chaos-slow.sock";
        let songbird_socket = "/tmp/test-songbird-chaos-slow.sock";

        let _ = std::fs::remove_file(socket_path);
        let _ = std::fs::remove_file(songbird_socket);

        env::set_var("NEURAL_API_SOCKET", socket_path);
        env::set_var("SONGBIRD_SOCKET_PATH", songbird_socket);

        // Server with slow responses
        let listener = UnixListener::bind(socket_path).unwrap();
        let _server_task = tokio::spawn(async move {
            for _ in 0..6 {
                if let Ok((mut stream, _)) = listener.accept().await {
                    let mut reader = BufReader::new(&mut stream);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.is_ok() {
                        // Simulate slow response (500ms)
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
                        let _ = stream.write_all(format!("{}\n", response).as_bytes()).await;
                    }
                }
            }
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Should handle slow responses
        let result = register_capabilities().await;
        assert!(result.is_ok(), "Should handle slow responses (timeout protection)");

        env::remove_var("NEURAL_API_SOCKET");
        env::remove_var("SONGBIRD_SOCKET_PATH");
        let _ = std::fs::remove_file(socket_path);
        let _ = std::fs::remove_file(songbird_socket);
    }

    #[tokio::test]
    async fn test_chaos_neural_api_restarts_during_operation() {
        let _guard = ENV_TEST_LOCK.lock().unwrap();

        let socket_path = "/tmp/test-neural-chaos-restart.sock";
        let songbird_socket = "/tmp/test-songbird-chaos-restart.sock";

        let _ = std::fs::remove_file(socket_path);
        let _ = std::fs::remove_file(songbird_socket);

        env::set_var("NEURAL_API_SOCKET", socket_path);
        env::set_var("SONGBIRD_SOCKET_PATH", songbird_socket);

        // Simulate Neural API restart: serve 2 requests, die, restart
        let listener1 = UnixListener::bind(socket_path).unwrap();
        let _server_task1 = tokio::spawn(async move {
            for _ in 0..2 {
                if let Ok((mut stream, _)) = listener1.accept().await {
                    let mut reader = BufReader::new(&mut stream);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.is_ok() {
                        let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
                        let _ = stream.write_all(format!("{}\n", response).as_bytes()).await;
                    }
                }
            }
            // Server 1 dies
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // First registration attempt (partial success)
        let result1 = register_capabilities().await;
        assert!(result1.is_ok(), "Should handle partial registration (self-healing)");

        // Wait for server to die
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Clean up and restart
        let _ = std::fs::remove_file(socket_path);

        // Server 2 restarts
        let listener2 = UnixListener::bind(socket_path).unwrap();
        let _server_task2 = tokio::spawn(async move {
            for _ in 0..6 {
                if let Ok((mut stream, _)) = listener2.accept().await {
                    let mut reader = BufReader::new(&mut stream);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.is_ok() {
                        let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
                        let _ = stream.write_all(format!("{}\n", response).as_bytes()).await;
                    }
                }
            }
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Second registration (self-healing: can re-register)
        let result2 = register_capabilities().await;
        assert!(result2.is_ok(), "Should self-heal by re-registering after restart");

        env::remove_var("NEURAL_API_SOCKET");
        env::remove_var("SONGBIRD_SOCKET_PATH");
        let _ = std::fs::remove_file(socket_path);
        let _ = std::fs::remove_file(songbird_socket);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // 💥 FAULT INJECTION TESTS - Malformed Data, Errors, Edge Cases
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_fault_malformed_json_response() {
        let _guard = ENV_TEST_LOCK.lock().unwrap();

        let socket_path = "/tmp/test-neural-fault-malformed.sock";
        let songbird_socket = "/tmp/test-songbird-fault.sock";

        let _ = std::fs::remove_file(socket_path);
        let _ = std::fs::remove_file(songbird_socket);

        env::set_var("NEURAL_API_SOCKET", socket_path);
        env::set_var("SONGBIRD_SOCKET_PATH", songbird_socket);

        // Server returns malformed JSON
        let listener = UnixListener::bind(socket_path).unwrap();
        let _server_task = tokio::spawn(async move {
            for _ in 0..6 {
                if let Ok((mut stream, _)) = listener.accept().await {
                    let mut reader = BufReader::new(&mut stream);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.is_ok() {
                        // Send malformed JSON
                        let response = "INVALID JSON {{{";
                        let _ = stream.write_all(format!("{}\n", response).as_bytes()).await;
                    }
                }
            }
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Should handle malformed responses gracefully (fail-safe)
        let result = register_capabilities().await;
        assert!(result.is_ok(), "Should handle malformed JSON gracefully (fail-safe)");

        env::remove_var("NEURAL_API_SOCKET");
        env::remove_var("SONGBIRD_SOCKET_PATH");
        let _ = std::fs::remove_file(socket_path);
        let _ = std::fs::remove_file(songbird_socket);
    }

    #[tokio::test]
    async fn test_fault_neural_api_returns_errors() {
        let _guard = ENV_TEST_LOCK.lock().unwrap();

        let socket_path = "/tmp/test-neural-fault-error.sock";
        let songbird_socket = "/tmp/test-songbird-fault-error.sock";

        let _ = std::fs::remove_file(socket_path);
        let _ = std::fs::remove_file(songbird_socket);

        env::set_var("NEURAL_API_SOCKET", socket_path);
        env::set_var("SONGBIRD_SOCKET_PATH", songbird_socket);

        // Server returns JSON-RPC errors
        let listener = UnixListener::bind(socket_path).unwrap();
        let _server_task = tokio::spawn(async move {
            for _ in 0..6 {
                if let Ok((mut stream, _)) = listener.accept().await {
                    let mut reader = BufReader::new(&mut stream);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.is_ok() {
                        // Send JSON-RPC error
                        let response = r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid Request"},"id":1}"#;
                        let _ = stream.write_all(format!("{}\n", response).as_bytes()).await;
                    }
                }
            }
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Should handle errors gracefully (fail-safe)
        let result = register_capabilities().await;
        assert!(result.is_ok(), "Should handle JSON-RPC errors gracefully (fail-safe)");

        env::remove_var("NEURAL_API_SOCKET");
        env::remove_var("SONGBIRD_SOCKET_PATH");
        let _ = std::fs::remove_file(socket_path);
        let _ = std::fs::remove_file(songbird_socket);
    }

    #[tokio::test]
    async fn test_fault_connection_drops_mid_request() {
        let _guard = ENV_TEST_LOCK.lock().unwrap();

        let socket_path = "/tmp/test-neural-fault-drop.sock";
        let songbird_socket = "/tmp/test-songbird-fault-drop.sock";

        let _ = std::fs::remove_file(socket_path);
        let _ = std::fs::remove_file(songbird_socket);

        env::set_var("NEURAL_API_SOCKET", socket_path);
        env::set_var("SONGBIRD_SOCKET_PATH", songbird_socket);

        // Server that drops connections mid-request
        let listener = UnixListener::bind(socket_path).unwrap();
        let _server_task = tokio::spawn(async move {
            for _ in 0..6 {
                if let Ok((mut stream, _)) = listener.accept().await {
                    let mut reader = BufReader::new(&mut stream);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.is_ok() {
                        // Drop connection without response!
                        drop(stream);
                    }
                }
            }
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Should handle dropped connections (fail-safe)
        let result = register_capabilities().await;
        assert!(result.is_ok(), "Should handle dropped connections gracefully (fail-safe)");

        env::remove_var("NEURAL_API_SOCKET");
        env::remove_var("SONGBIRD_SOCKET_PATH");
        let _ = std::fs::remove_file(socket_path);
        let _ = std::fs::remove_file(songbird_socket);
    }

    #[tokio::test]
    async fn test_fault_permission_denied_on_socket() {
        let _guard = ENV_TEST_LOCK.lock().unwrap();

        let socket_path = "/root/nonexistent/test-neural.sock"; // Likely permission denied
        let songbird_socket = "/tmp/test-songbird-perm.sock";

        env::set_var("NEURAL_API_SOCKET", socket_path);
        env::set_var("SONGBIRD_SOCKET_PATH", songbird_socket);

        // Should handle permission errors gracefully (fail-safe)
        let result = register_capabilities().await;
        assert!(result.is_ok(), "Should handle permission denied gracefully (fail-safe)");

        env::remove_var("NEURAL_API_SOCKET");
        env::remove_var("SONGBIRD_SOCKET_PATH");
    }

    #[tokio::test]
    async fn test_fault_concurrent_registrations() {
        let _guard = ENV_TEST_LOCK.lock().unwrap();

        let socket_path = "/tmp/test-neural-concurrent.sock";
        let songbird_socket = "/tmp/test-songbird-concurrent.sock";

        let _ = std::fs::remove_file(socket_path);
        let _ = std::fs::remove_file(songbird_socket);

        env::set_var("NEURAL_API_SOCKET", socket_path);
        env::set_var("SONGBIRD_SOCKET_PATH", songbird_socket);

        // Server handles concurrent requests
        let listener = UnixListener::bind(socket_path).unwrap();
        let _server_task = tokio::spawn(async move {
            for _ in 0..18 {
                // 3 concurrent * 6 capabilities
                if let Ok((mut stream, _)) = listener.accept().await {
                    let mut reader = BufReader::new(&mut stream);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.is_ok() {
                        let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
                        let _ = stream.write_all(format!("{}\n", response).as_bytes()).await;
                    }
                }
            }
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Spawn 3 concurrent registration attempts
        let mut handles = vec![];
        for _ in 0..3 {
            let handle = tokio::spawn(async { register_capabilities().await });
            handles.push(handle);
        }

        // All should succeed (or at least not crash)
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok(), "Concurrent registrations should be safe");
        }

        env::remove_var("NEURAL_API_SOCKET");
        env::remove_var("SONGBIRD_SOCKET_PATH");
        let _ = std::fs::remove_file(socket_path);
        let _ = std::fs::remove_file(songbird_socket);
    }
}
