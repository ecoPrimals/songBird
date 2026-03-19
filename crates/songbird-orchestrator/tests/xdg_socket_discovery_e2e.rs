//! XDG Socket Discovery E2E Tests
//!
//! **Created**: February 4, 2026
//! **Purpose**: Validate XDG-compliant socket discovery per PRIMAL_DEPLOYMENT_STANDARD
//!
//! ## Test Coverage
//!
//! 1. **XDG Path Priority**: XDG_RUNTIME_DIR > /tmp/biomeos > /tmp (legacy)
//! 2. **Socket Naming**: {primal}.sock (no family suffix)
//! 3. **Cross-Primal Discovery**: Songbird finds BearDog, Neural API at XDG paths
//! 4. **Fallback Chain**: Graceful degradation to legacy paths
//!
//! ## PRIMAL_DEPLOYMENT_STANDARD Compliance
//!
//! ```text
//! Path Priority:
//! 1. $PRIMAL_SOCKET env var (explicit override)
//! 2. $XDG_RUNTIME_DIR/biomeos/{primal}.sock (XDG-compliant)
//! 3. /tmp/biomeos/{primal}.sock (fallback)
//! 4. /tmp/{primal}.sock (legacy)
//!
//! Socket Naming:
//! - beardog.sock (NOT beardog-nat0.sock)
//! - songbird.sock (NOT songbird-nat0.sock)
//! - neural-api.sock (NOT neural-api-nat0.sock)
//! ```

use std::env;
use std::path::PathBuf;
use std::sync::Mutex;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::oneshot;

// Mutex to serialize tests that modify environment variables
static ENV_MUTEX: Mutex<()> = Mutex::new(());

/// Helper to create a mock socket server
async fn create_mock_socket(path: &std::path::Path) -> UnixListener {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    // Remove existing socket
    let _ = std::fs::remove_file(path);
    UnixListener::bind(path).unwrap()
}

/// Helper to run mock JSON-RPC server with readiness signal.
/// Returns a receiver that resolves when the server is ready to accept connections.
fn run_mock_jsonrpc_server(
    listener: UnixListener,
    response: &'static str,
) -> oneshot::Receiver<()> {
    let (ready_tx, ready_rx) = oneshot::channel();
    tokio::spawn(async move {
        let _ = ready_tx.send(());
        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut reader = BufReader::new(&mut stream);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_ok() {
                    let _ = stream.write_all(format!("{}\n", response).as_bytes()).await;
                }
            }
        }
    });
    ready_rx
}

// ═══════════════════════════════════════════════════════════════════════════
// 🧪 E2E: XDG PATH PRIORITY TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_e2e_xdg_path_priority_xdg_runtime_dir_first() {
    let _guard = ENV_MUTEX.lock().unwrap();

    // Setup: Create sockets at all three locations
    let temp_dir = env::temp_dir().join("test-xdg-priority-e2e");
    let xdg_biomeos = temp_dir.join("biomeos");
    let tmp_biomeos = PathBuf::from("/tmp/biomeos");

    std::fs::create_dir_all(&xdg_biomeos).unwrap();
    std::fs::create_dir_all(&tmp_biomeos).unwrap();

    // Create socket files at all locations
    let xdg_socket = xdg_biomeos.join("beardog.sock");
    let tmp_biomeos_socket = tmp_biomeos.join("beardog.sock");
    let legacy_socket = PathBuf::from("/tmp/beardog.sock");

    std::fs::write(&xdg_socket, "xdg").unwrap();
    std::fs::write(&tmp_biomeos_socket, "tmp_biomeos").unwrap();
    std::fs::write(&legacy_socket, "legacy").unwrap();

    // Clear env vars
    env::remove_var("CRYPTO_PROVIDER_SOCKET");
    env::remove_var("BEARDOG_SOCKET");
    env::set_var("XDG_RUNTIME_DIR", temp_dir.to_str().unwrap());

    // Test: Discovery should find XDG socket first
    use songbird_orchestrator::primal_discovery::{discover, Capability};
    let result = discover(Capability::Crypto).await;

    assert!(result.is_ok(), "Should discover socket");
    let found = result.unwrap();
    assert!(
        found.contains(&temp_dir.to_string_lossy().to_string())
            && found.contains("biomeos/beardog.sock"),
        "Should prioritize XDG_RUNTIME_DIR socket, got: {}",
        found
    );

    // Cleanup
    env::remove_var("XDG_RUNTIME_DIR");
    let _ = std::fs::remove_dir_all(&temp_dir);
    let _ = std::fs::remove_file(&tmp_biomeos_socket);
    let _ = std::fs::remove_file(&legacy_socket);
}

#[tokio::test]
async fn test_e2e_fallback_to_tmp_biomeos_when_no_xdg() {
    let _guard = ENV_MUTEX.lock().unwrap();

    // Clear XDG
    env::remove_var("XDG_RUNTIME_DIR");
    env::remove_var("CRYPTO_PROVIDER_SOCKET");
    env::remove_var("BEARDOG_SOCKET");

    // Create /tmp/biomeos socket
    let tmp_biomeos = PathBuf::from("/tmp/biomeos");
    std::fs::create_dir_all(&tmp_biomeos).unwrap();
    let socket = tmp_biomeos.join("beardog.sock");
    std::fs::write(&socket, "").unwrap();

    // Remove legacy socket to ensure fallback works
    let _ = std::fs::remove_file("/tmp/beardog.sock");

    use songbird_orchestrator::primal_discovery::{discover, Capability};
    let result = discover(Capability::Crypto).await;

    assert!(result.is_ok(), "Should discover /tmp/biomeos socket");
    let found = result.unwrap();
    assert!(
        found == "/tmp/biomeos/beardog.sock",
        "Should use /tmp/biomeos fallback, got: {}",
        found
    );

    // Cleanup
    let _ = std::fs::remove_file(&socket);
}

// ═══════════════════════════════════════════════════════════════════════════
// 🧪 E2E: SOCKET NAMING TESTS (No Family Suffix)
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_e2e_socket_naming_no_family_suffix() {
    let _guard = ENV_MUTEX.lock().unwrap();

    let temp_dir = env::temp_dir().join("test-socket-naming-e2e");
    let biomeos_dir = temp_dir.join("biomeos");
    std::fs::create_dir_all(&biomeos_dir).unwrap();

    // Create correct socket (no family suffix)
    let correct_socket = biomeos_dir.join("beardog.sock");
    std::fs::write(&correct_socket, "").unwrap();

    // Create legacy socket (with family suffix) - should NOT be used if correct exists
    let legacy_socket = biomeos_dir.join("beardog-nat0.sock");
    std::fs::write(&legacy_socket, "").unwrap();

    env::set_var("XDG_RUNTIME_DIR", temp_dir.to_str().unwrap());
    env::remove_var("CRYPTO_PROVIDER_SOCKET");
    env::remove_var("BEARDOG_SOCKET");

    use songbird_orchestrator::primal_discovery::{discover, Capability};
    let result = discover(Capability::Crypto).await;

    assert!(result.is_ok());
    let found = result.unwrap();

    // Should find beardog.sock, NOT beardog-nat0.sock
    assert!(
        found.ends_with("beardog.sock") && !found.contains("-nat0"),
        "Should use {{primal}}.sock naming (no family suffix), got: {}",
        found
    );

    // Cleanup
    env::remove_var("XDG_RUNTIME_DIR");
    let _ = std::fs::remove_dir_all(&temp_dir);
}

// ═══════════════════════════════════════════════════════════════════════════
// 🧪 E2E: CROSS-PRIMAL DISCOVERY TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_e2e_discover_all_primals_at_xdg() {
    let _guard = ENV_MUTEX.lock().unwrap();

    let temp_dir = env::temp_dir().join("test-all-primals-e2e");
    let biomeos_dir = temp_dir.join("biomeos");
    std::fs::create_dir_all(&biomeos_dir).unwrap();

    // Create sockets for all primals
    let primals = ["beardog", "songbird", "squirrel", "nestgate", "messenger"];
    for primal in &primals {
        let socket = biomeos_dir.join(format!("{}.sock", primal));
        std::fs::write(&socket, "").unwrap();
    }

    env::set_var("XDG_RUNTIME_DIR", temp_dir.to_str().unwrap());

    // Clear all env vars
    env::remove_var("CRYPTO_PROVIDER_SOCKET");
    env::remove_var("BEARDOG_SOCKET");
    env::remove_var("HTTP_PROVIDER_SOCKET");
    env::remove_var("AI_PROVIDER_SOCKET");
    env::remove_var("STORAGE_PROVIDER_SOCKET");
    env::remove_var("MESSAGING_PROVIDER_SOCKET");

    use songbird_orchestrator::primal_discovery::{discover, Capability};

    // Test each capability
    let capabilities = [
        (Capability::Crypto, "beardog.sock"),
        (Capability::Http, "songbird.sock"),
        (Capability::Ai, "squirrel.sock"),
        (Capability::Storage, "nestgate.sock"),
        (Capability::Messaging, "messenger.sock"),
    ];

    for (cap, expected_name) in &capabilities {
        let result = discover(*cap).await;
        assert!(result.is_ok(), "{:?} discovery should succeed", cap);
        let found = result.unwrap();
        assert!(
            found.ends_with(expected_name),
            "{:?} should find {}, got: {}",
            cap,
            expected_name,
            found
        );
    }

    // Cleanup
    env::remove_var("XDG_RUNTIME_DIR");
    let _ = std::fs::remove_dir_all(&temp_dir);
}

// ═══════════════════════════════════════════════════════════════════════════
// 🧪 E2E: CAPABILITY REGISTRATION WITH XDG
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_e2e_capability_registration_discovers_xdg_neural_api() {
    let _guard = ENV_MUTEX.lock().unwrap();

    let temp_dir = env::temp_dir().join("test-cap-reg-e2e");
    let biomeos_dir = temp_dir.join("biomeos");
    std::fs::create_dir_all(&biomeos_dir).unwrap();

    let neural_socket_path = biomeos_dir.join("neural-api.sock");
    let songbird_socket_path = biomeos_dir.join("songbird.sock");

    // Start mock Neural API server with readiness signal
    let listener = create_mock_socket(&neural_socket_path).await;
    let ready_rx = run_mock_jsonrpc_server(listener, r#"{"jsonrpc":"2.0","result":"ok","id":1}"#);
    ready_rx.await.expect("Mock server failed to signal readiness");

    env::set_var("XDG_RUNTIME_DIR", temp_dir.to_str().unwrap());
    env::set_var("SONGBIRD_SOCKET_PATH", songbird_socket_path.to_str().unwrap());
    env::remove_var("NEURAL_API_SOCKET"); // Let it discover via XDG

    use songbird_orchestrator::capability_registration::register_capabilities;
    let result = register_capabilities().await;

    // Should succeed using XDG-discovered Neural API socket
    assert!(result.is_ok(), "Registration should succeed with XDG Neural API");

    // Cleanup
    env::remove_var("XDG_RUNTIME_DIR");
    env::remove_var("SONGBIRD_SOCKET_PATH");
    let _ = std::fs::remove_dir_all(&temp_dir);
}

// ═══════════════════════════════════════════════════════════════════════════
// 🧪 E2E: SECURITY CLIENT WITH XDG BEARDOG
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_e2e_security_client_discovers_xdg_beardog() {
    let _guard = ENV_MUTEX.lock().unwrap();

    let temp_dir = env::temp_dir().join("test-sec-client-e2e");
    let biomeos_dir = temp_dir.join("biomeos");
    std::fs::create_dir_all(&biomeos_dir).unwrap();

    let beardog_socket = biomeos_dir.join("beardog.sock");
    std::fs::write(&beardog_socket, "").unwrap();

    env::set_var("XDG_RUNTIME_DIR", temp_dir.to_str().unwrap());
    env::remove_var("CRYPTO_PROVIDER_SOCKET");
    env::remove_var("BEARDOG_SOCKET");

    // Test that crypto discovery finds XDG socket
    use songbird_orchestrator::crypto::discovery::get_beardog_crypto_socket;
    let result = get_beardog_crypto_socket().await;

    assert!(result.is_ok(), "Should discover BearDog at XDG path");
    let found = result.unwrap();
    assert!(found.contains("biomeos/beardog.sock"), "Should use XDG BearDog path, got: {}", found);

    // Cleanup
    env::remove_var("XDG_RUNTIME_DIR");
    let _ = std::fs::remove_dir_all(&temp_dir);
}

// ═══════════════════════════════════════════════════════════════════════════
// 🧪 E2E: DIRECTORY STRUCTURE COMPLIANCE
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_e2e_xdg_directory_structure_compliance() {
    let _guard = ENV_MUTEX.lock().unwrap();

    // Verify expected directory structure
    // /run/user/$UID/biomeos/
    // ├── beardog.sock
    // ├── songbird.sock
    // ├── neural-api.sock
    // └── ...

    let temp_dir = env::temp_dir().join("test-dir-structure-e2e");
    let biomeos_dir = temp_dir.join("biomeos");
    std::fs::create_dir_all(&biomeos_dir).unwrap();

    // Verify we're creating sockets in the right structure
    let expected_sockets = [
        "beardog.sock",
        "songbird.sock",
        "neural-api.sock",
        "squirrel.sock",
        "nestgate.sock",
        "messenger.sock",
    ];

    for socket_name in &expected_sockets {
        let socket_path = biomeos_dir.join(socket_name);
        std::fs::write(&socket_path, "").unwrap();
        assert!(socket_path.exists(), "Socket {} should exist in biomeos dir", socket_name);
    }

    // Verify structure is correct
    let entries: Vec<_> = std::fs::read_dir(&biomeos_dir).unwrap().filter_map(|e| e.ok()).collect();

    assert_eq!(
        entries.len(),
        expected_sockets.len(),
        "Should have exactly {} sockets in biomeos dir",
        expected_sockets.len()
    );

    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        assert!(name.ends_with(".sock"), "All entries should be .sock files");
        assert!(!name.contains("-nat0"), "No sockets should have -nat0 suffix");
    }

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}

// ═══════════════════════════════════════════════════════════════════════════
// 🧪 E2E: ENV VAR OVERRIDE TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_e2e_env_var_overrides_xdg() {
    let _guard = ENV_MUTEX.lock().unwrap();

    let temp_dir = env::temp_dir().join("test-env-override-e2e");
    let biomeos_dir = temp_dir.join("biomeos");
    std::fs::create_dir_all(&biomeos_dir).unwrap();

    // Create XDG socket
    let xdg_socket = biomeos_dir.join("beardog.sock");
    std::fs::write(&xdg_socket, "xdg").unwrap();

    // Create custom override socket
    let custom_socket = temp_dir.join("custom-beardog.sock");
    std::fs::write(&custom_socket, "custom").unwrap();

    env::set_var("XDG_RUNTIME_DIR", temp_dir.to_str().unwrap());
    env::set_var("CRYPTO_PROVIDER_SOCKET", custom_socket.to_str().unwrap());

    use songbird_orchestrator::primal_discovery::{discover, Capability};
    let result = discover(Capability::Crypto).await;

    assert!(result.is_ok());
    let found = result.unwrap();

    // Env var should take priority over XDG
    assert_eq!(found, custom_socket.to_str().unwrap(), "Env var should override XDG discovery");

    // Cleanup
    env::remove_var("XDG_RUNTIME_DIR");
    env::remove_var("CRYPTO_PROVIDER_SOCKET");
    let _ = std::fs::remove_dir_all(&temp_dir);
}
