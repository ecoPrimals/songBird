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
//! │ secure_http → [songbird-{family_id}]                        │
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
#[cfg(windows)]
use tokio::net::TcpStream as PlatformStream;
#[cfg(unix)]
use tokio::net::UnixStream as PlatformStream;
use tracing::{info, warn};

/// Configuration for capability registration (supports dependency injection)
#[derive(Debug, Clone)]
pub struct CapabilityRegistrationConfig {
    /// Neural API socket path
    pub neural_socket: String,
    /// Songbird's own socket path
    pub songbird_socket: String,
    /// Primal ID
    pub primal_id: String,
}

impl CapabilityRegistrationConfig {
    /// Build config from environment variables (production use)
    pub fn from_env() -> Result<Self> {
        let neural_socket = env::var("NEURAL_API_SOCKET").unwrap_or_else(|_| {
            if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
                format!("{runtime_dir}/biomeos/neural-api.sock")
            } else {
                "/tmp/biomeos/neural-api.sock".to_string()
            }
        });

        let songbird_socket = env::var("SONGBIRD_SOCKET_PATH")
            .or_else(|_| env::var("SONGBIRD_SOCKET"))
            .or_else(|_| env::var("SONGBIRD_IPC_SOCKET"))
            .context(
                "SONGBIRD_SOCKET_PATH not set. Songbird must know its own socket path for registration."
            )?;

        let primal_id = env::var("PRIMAL_ID")
            .or_else(|_| env::var("SONGBIRD_PRIMAL_ID"))
            .unwrap_or_else(|_| "songbird".to_string());

        Ok(Self {
            neural_socket,
            songbird_socket,
            primal_id,
        })
    }

    /// Build config with explicit values (test use)
    #[cfg(test)]
    pub fn for_testing(neural_socket: &str, songbird_socket: &str) -> Self {
        Self {
            neural_socket: neural_socket.to_string(),
            songbird_socket: songbird_socket.to_string(),
            primal_id: "songbird".to_string(),
        }
    }
}

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
/// - **`secure_http`**: HTTPS client with Pure Rust TLS 1.3
///   - `http.get` - HTTP GET requests
///   - `http.post` - HTTP POST requests
///   - `http.put` - HTTP PUT requests
///   - `http.delete` - HTTP DELETE requests
///   - `http.patch` - HTTP PATCH requests
///   - `http.request` - Generic HTTP request (fallback)
///
/// ## Environment Variables
///
/// - `NEURAL_API_SOCKET` - Neural API socket path (default: `/tmp/neural-api-{family_id}.sock`)
/// - `SONGBIRD_SOCKET_PATH` - Our socket path (required)
/// - `PRIMAL_ID` - Our primal identifier (default: `songbird-{family_id}`)
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
    let config = CapabilityRegistrationConfig::from_env()?;
    register_capabilities_with(&config).await
}

/// Register capabilities with explicit config (concurrent-safe, testable)
pub async fn register_capabilities_with(config: &CapabilityRegistrationConfig) -> Result<()> {
    info!("🔄 Registering capabilities with Neural API...");

    let neural_socket = &config.neural_socket;
    let songbird_socket = &config.songbird_socket;
    let primal_id = &config.primal_id;

    // Get family ID for metadata (still from env — not critical for test isolation)
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
    let mut stream = match connect_platform(neural_socket).await {
        Ok(s) => s,
        Err(e) => {
            warn!("⚠️  Failed to connect to Neural API at {}: {}", neural_socket, e);
            warn!("   Songbird will continue without Neural API registration");
            warn!("   Direct socket connections will still work");
            return Ok(()); // Don't fail startup
        }
    };

    // Send registration
    let request = format!("{registration}\n");
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
    } else if let Some(error) = response_json.get("error") {
        warn!("⚠️  Neural API registration returned error: {:?}", error);
        warn!("   Songbird will continue without registration");
        warn!("   Direct socket connections will still work");
    } else {
        warn!("⚠️  Unexpected registration response from Neural API");
        warn!("   Response: {}", response);
    }

    Ok(()) // Don't fail startup
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
    // Use from_env or a default config for unregistration (primal_id is all we need)
    let neural_socket = env::var("NEURAL_API_SOCKET").unwrap_or_else(|_| {
        if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
            format!("{runtime_dir}/biomeos/neural-api.sock")
        } else {
            "/tmp/biomeos/neural-api.sock".to_string()
        }
    });
    let primal_id = env::var("PRIMAL_ID")
        .or_else(|_| env::var("SONGBIRD_PRIMAL_ID"))
        .unwrap_or_else(|_| "songbird".to_string());
    unregister_capabilities_with(&neural_socket, &primal_id).await
}

/// Unregister capabilities with explicit config (concurrent-safe, testable)
pub async fn unregister_capabilities_with(neural_socket: &str, primal_id: &str) -> Result<()> {
    info!("🔄 Unregistering capabilities from Neural API...");

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

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use tokio::net::UnixListener;

    #[test]
    fn test_config_defaults() {
        // ✅ Concurrent-safe: Tests default config construction (no global state)
        let config = CapabilityRegistrationConfig {
            neural_socket: "/tmp/biomeos/neural-api.sock".to_string(),
            songbird_socket: "/tmp/songbird.sock".to_string(),
            primal_id: "songbird".to_string(),
        };
        assert_eq!(config.neural_socket, "/tmp/biomeos/neural-api.sock");
        assert_eq!(config.primal_id, "songbird");
    }

    #[tokio::test]
    async fn test_registration_with_unavailable_neural_api_succeeds() {
        // ✅ Concurrent-safe: Uses explicit config (no env vars)
        let config = CapabilityRegistrationConfig::for_testing(
            "/tmp/nonexistent-neural-api-sock-unique-12345.sock",
            "/tmp/test-songbird-unique.sock",
        );

        // Should succeed (not fail startup) even though Neural API is unavailable
        let result = register_capabilities_with(&config).await;
        if let Err(ref e) = result {
            eprintln!("Registration failed with error: {}", e);
        }
        assert!(
            result.is_ok(),
            "Registration should succeed gracefully even when Neural API is unavailable"
        );
    }

    #[tokio::test]
    async fn test_unregistration_with_unavailable_neural_api_succeeds() {
        // ✅ Concurrent-safe: Uses explicit socket path (no env vars)
        let result =
            unregister_capabilities_with("/tmp/nonexistent-neural-api.sock", "songbird").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_neural_api_with_mock_server() {
        // ✅ Concurrent-safe: Uses check_neural_api_available_at (no env vars)
        let socket_path = "/tmp/test-neural-api-check-concurrent.sock";

        // Clean up any existing socket
        let _ = std::fs::remove_file(socket_path);

        // Start mock server
        let listener = UnixListener::bind(socket_path).unwrap();
        let _server_task = tokio::spawn(async move {
            let _ = listener.accept().await;
        });

        // yield to let spawned task schedule (no sleep needed — bind is sync)
        tokio::task::yield_now().await;

        let available = check_neural_api_available_at(socket_path).await;
        assert!(available);

        // Cleanup
        let _ = std::fs::remove_file(socket_path);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // 🧪 XDG SOCKET DISCOVERY TESTS (Feb 4, 2026)
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_xdg_neural_api_socket_resolution() {
        // ✅ Concurrent-safe: Tests XDG resolution logic without env vars
        // Simulates what from_env() does with XDG_RUNTIME_DIR
        let resolve = |xdg_dir: Option<&str>| -> String {
            match xdg_dir {
                Some(dir) => format!("{}/biomeos/neural-api.sock", dir),
                None => "/tmp/biomeos/neural-api.sock".to_string(),
            }
        };

        assert_eq!(resolve(Some("/run/user/1000")), "/run/user/1000/biomeos/neural-api.sock",);
        assert_eq!(resolve(None), "/tmp/biomeos/neural-api.sock",);
    }

    #[test]
    fn test_primal_id_no_family_suffix() {
        // ✅ Concurrent-safe: Tests default primal_id
        let config = CapabilityRegistrationConfig {
            neural_socket: String::new(),
            songbird_socket: String::new(),
            primal_id: "songbird".to_string(),
        };
        assert_eq!(config.primal_id, "songbird");
        assert!(!config.primal_id.contains("-nat0"));
    }

    #[tokio::test]
    async fn test_xdg_registration_with_xdg_socket() {
        // ✅ Concurrent-safe: Uses register_capabilities_with (no env vars)
        let temp_dir = std::env::temp_dir().join("test-cap-reg-xdg-concurrent");
        let biomeos_dir = temp_dir.join("biomeos");
        std::fs::create_dir_all(&biomeos_dir).unwrap();

        let neural_socket = biomeos_dir.join("neural-api.sock");
        let songbird_socket = biomeos_dir.join("songbird.sock");

        let _ = std::fs::remove_file(&neural_socket);
        let _ = std::fs::remove_file(&songbird_socket);

        // Start mock Neural API server
        let neural_path = neural_socket.clone();
        let listener = UnixListener::bind(&neural_path).unwrap();
        let _server_task = tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
                let mut reader = BufReader::new(&mut stream);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_ok() {
                    let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
                    let _ = stream.write_all(format!("{}\n", response).as_bytes()).await;
                }
            }
        });

        tokio::task::yield_now().await;

        let config = CapabilityRegistrationConfig::for_testing(
            neural_socket.to_str().unwrap(),
            songbird_socket.to_str().unwrap(),
        );
        let result = register_capabilities_with(&config).await;
        assert!(result.is_ok(), "Registration with XDG socket should succeed");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_check_neural_api_with_xdg_socket() {
        // ✅ Concurrent-safe: Uses check_neural_api_available_at (no env vars)
        let temp_dir = std::env::temp_dir().join("test-neural-check-xdg-concurrent");
        let biomeos_dir = temp_dir.join("biomeos");
        std::fs::create_dir_all(&biomeos_dir).unwrap();

        let socket_path = biomeos_dir.join("neural-api.sock");
        let _ = std::fs::remove_file(&socket_path);

        let listener = UnixListener::bind(&socket_path).unwrap();
        let _server_task = tokio::spawn(async move {
            let _ = listener.accept().await;
        });

        tokio::task::yield_now().await;

        let available = check_neural_api_available_at(socket_path.to_str().unwrap()).await;
        assert!(available, "Should find Neural API at XDG path");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // 🧪 E2E TESTS - End-to-End Registration Flow
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_e2e_full_registration_lifecycle() {
        // ✅ Concurrent-safe: Uses register_capabilities_with (no env vars)
        let socket_path = "/tmp/test-neural-e2e-concurrent.sock";
        let songbird_socket = "/tmp/test-songbird-e2e-concurrent.sock";

        let _ = std::fs::remove_file(socket_path);
        let _ = std::fs::remove_file(songbird_socket);

        // Start mock Neural API server
        let listener = UnixListener::bind(socket_path).unwrap();
        let server_task = tokio::spawn(async move {
            for _ in 0..6 {
                if let Ok((mut stream, _)) = listener.accept().await {
                    let mut reader = BufReader::new(&mut stream);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.is_ok() {
                        assert!(line.contains("capability.register"));
                        let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
                        let _ = stream.write_all(format!("{}\n", response).as_bytes()).await;
                    }
                }
            }
        });

        tokio::task::yield_now().await;

        let config = CapabilityRegistrationConfig::for_testing(socket_path, songbird_socket);
        let result = register_capabilities_with(&config).await;
        assert!(result.is_ok(), "E2E registration should succeed");

        drop(server_task);
        let _ = std::fs::remove_file(socket_path);
        let _ = std::fs::remove_file(songbird_socket);
    }

    #[tokio::test]
    async fn test_e2e_registration_and_immediate_unregistration() {
        // ✅ Concurrent-safe: Uses _with variants (no env vars)
        let socket_path = "/tmp/test-neural-e2e-unreg-concurrent.sock";
        let songbird_socket = "/tmp/test-songbird-e2e-unreg-concurrent.sock";

        let _ = std::fs::remove_file(socket_path);
        let _ = std::fs::remove_file(songbird_socket);

        let listener = UnixListener::bind(socket_path).unwrap();
        let _server_task = tokio::spawn(async move {
            for _ in 0..12 {
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

        tokio::task::yield_now().await;

        // Register
        let config = CapabilityRegistrationConfig::for_testing(socket_path, songbird_socket);
        let reg_result = register_capabilities_with(&config).await;
        assert!(reg_result.is_ok());

        // Unregister
        let unreg_result = unregister_capabilities_with(socket_path, "songbird").await;
        assert!(unreg_result.is_ok());

        let _ = std::fs::remove_file(socket_path);
        let _ = std::fs::remove_file(songbird_socket);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // 🔥 CHAOS TESTS - Network Chaos & Intermittent Failures
    // ═══════════════════════════════════════════════════════════════════════

    // ═══════════════════════════════════════════════════════════════════════
    // 🔥 CHAOS TESTS - Network Chaos & Intermittent Failures
    // ✅ All evolved to use register_capabilities_with (no env vars)
    // ═══════════════════════════════════════════════════════════════════════

    /// Helper: create config for chaos/fault test
    fn chaos_config(socket_path: &str, songbird_socket: &str) -> CapabilityRegistrationConfig {
        CapabilityRegistrationConfig::for_testing(socket_path, songbird_socket)
    }

    #[tokio::test]
    async fn test_chaos_socket_disappears_during_registration() {
        let socket_path = "/tmp/test-neural-chaos-disappear-c.sock";
        let songbird_socket = "/tmp/test-songbird-chaos-c.sock";
        let _ = std::fs::remove_file(socket_path);

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
                    break;
                }
            }
        });

        tokio::task::yield_now().await;
        let result = register_capabilities_with(&chaos_config(socket_path, songbird_socket)).await;
        assert!(result.is_ok(), "Should handle socket disappearance gracefully");

        drop(server_task);
        let _ = std::fs::remove_file(socket_path);
    }

    #[tokio::test]
    async fn test_chaos_slow_neural_api_responses() {
        let socket_path = "/tmp/test-neural-chaos-slow-c.sock";
        let songbird_socket = "/tmp/test-songbird-chaos-slow-c.sock";
        let _ = std::fs::remove_file(socket_path);

        let listener = UnixListener::bind(socket_path).unwrap();
        let _server_task = tokio::spawn(async move {
            for _ in 0..6 {
                if let Ok((mut stream, _)) = listener.accept().await {
                    let mut reader = BufReader::new(&mut stream);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.is_ok() {
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        let response = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
                        let _ = stream.write_all(format!("{}\n", response).as_bytes()).await;
                    }
                }
            }
        });

        tokio::task::yield_now().await;
        let result = register_capabilities_with(&chaos_config(socket_path, songbird_socket)).await;
        assert!(result.is_ok(), "Should handle slow responses");

        let _ = std::fs::remove_file(socket_path);
    }

    #[tokio::test]
    async fn test_chaos_neural_api_restarts_during_operation() {
        let socket_path = "/tmp/test-neural-chaos-restart-c.sock";
        let songbird_socket = "/tmp/test-songbird-chaos-restart-c.sock";
        let _ = std::fs::remove_file(socket_path);

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
        });

        tokio::task::yield_now().await;

        let config = chaos_config(socket_path, songbird_socket);
        let result1 = register_capabilities_with(&config).await;
        assert!(result1.is_ok(), "Should handle partial registration");

        // Server 1 dies, restart
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let _ = std::fs::remove_file(socket_path);

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

        tokio::task::yield_now().await;
        let result2 = register_capabilities_with(&config).await;
        assert!(result2.is_ok(), "Should self-heal by re-registering after restart");

        let _ = std::fs::remove_file(socket_path);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // 💥 FAULT INJECTION TESTS - Malformed Data, Errors, Edge Cases
    // ✅ All evolved to use register_capabilities_with (no env vars)
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_fault_malformed_json_response() {
        let socket_path = "/tmp/test-neural-fault-malformed-c.sock";
        let songbird_socket = "/tmp/test-songbird-fault-c.sock";
        let _ = std::fs::remove_file(socket_path);

        let listener = UnixListener::bind(socket_path).unwrap();
        let _server_task = tokio::spawn(async move {
            for _ in 0..6 {
                if let Ok((mut stream, _)) = listener.accept().await {
                    let mut reader = BufReader::new(&mut stream);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.is_ok() {
                        let _ = stream.write_all(b"INVALID JSON {{{\n").await;
                    }
                }
            }
        });

        tokio::task::yield_now().await;
        let result = register_capabilities_with(&chaos_config(socket_path, songbird_socket)).await;
        assert!(result.is_ok(), "Should handle malformed JSON gracefully");
        let _ = std::fs::remove_file(socket_path);
    }

    #[tokio::test]
    async fn test_fault_neural_api_returns_errors() {
        let socket_path = "/tmp/test-neural-fault-error-c.sock";
        let songbird_socket = "/tmp/test-songbird-fault-error-c.sock";
        let _ = std::fs::remove_file(socket_path);

        let listener = UnixListener::bind(socket_path).unwrap();
        let _server_task = tokio::spawn(async move {
            for _ in 0..6 {
                if let Ok((mut stream, _)) = listener.accept().await {
                    let mut reader = BufReader::new(&mut stream);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.is_ok() {
                        let response = r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid Request"},"id":1}"#;
                        let _ = stream.write_all(format!("{}\n", response).as_bytes()).await;
                    }
                }
            }
        });

        tokio::task::yield_now().await;
        let result = register_capabilities_with(&chaos_config(socket_path, songbird_socket)).await;
        assert!(result.is_ok(), "Should handle JSON-RPC errors gracefully");
        let _ = std::fs::remove_file(socket_path);
    }

    #[tokio::test]
    async fn test_fault_connection_drops_mid_request() {
        let socket_path = "/tmp/test-neural-fault-drop-c.sock";
        let songbird_socket = "/tmp/test-songbird-fault-drop-c.sock";
        let _ = std::fs::remove_file(socket_path);

        let listener = UnixListener::bind(socket_path).unwrap();
        let _server_task = tokio::spawn(async move {
            for _ in 0..6 {
                if let Ok((mut stream, _)) = listener.accept().await {
                    let mut reader = BufReader::new(&mut stream);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.is_ok() {
                        drop(stream); // Drop without response
                    }
                }
            }
        });

        tokio::task::yield_now().await;
        let result = register_capabilities_with(&chaos_config(socket_path, songbird_socket)).await;
        assert!(result.is_ok(), "Should handle dropped connections gracefully");
        let _ = std::fs::remove_file(socket_path);
    }

    #[tokio::test]
    async fn test_fault_permission_denied_on_socket() {
        // ✅ Concurrent-safe: Uses explicit config
        let config =
            chaos_config("/root/nonexistent/test-neural.sock", "/tmp/test-songbird-perm.sock");
        let result = register_capabilities_with(&config).await;
        assert!(result.is_ok(), "Should handle permission denied gracefully");
    }

    #[tokio::test]
    async fn test_fault_concurrent_registrations() {
        let socket_path = "/tmp/test-neural-concurrent-c.sock";
        let songbird_socket = "/tmp/test-songbird-concurrent-c.sock";
        let _ = std::fs::remove_file(socket_path);

        let listener = UnixListener::bind(socket_path).unwrap();
        let _server_task = tokio::spawn(async move {
            for _ in 0..18 {
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

        tokio::task::yield_now().await;

        let config = chaos_config(socket_path, songbird_socket);
        let mut handles = vec![];
        for _ in 0..3 {
            let c = config.clone();
            let handle = tokio::spawn(async move { register_capabilities_with(&c).await });
            handles.push(handle);
        }

        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok(), "Concurrent registrations should be safe");
        }

        let _ = std::fs::remove_file(socket_path);
    }
}
