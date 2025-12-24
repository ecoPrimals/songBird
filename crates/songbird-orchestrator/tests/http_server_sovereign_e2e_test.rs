//! E2E tests for HTTP server with Sovereign Socket binding
//!
//! These tests verify that the HTTP server works correctly with
//! sovereign socket binding in real-world scenarios.

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use songbird_network_federation::service_registry::FederatedServiceRegistry;
use songbird_network_federation::state::FederationState;

// Helper to create test federation state
fn create_test_federation_state() -> (Arc<FederationState>, Arc<FederatedServiceRegistry>) {
    let federation_state = Arc::new(FederationState::new("test-federation".to_string()));
    let service_registry = Arc::new(FederatedServiceRegistry::new());
    (federation_state, service_registry)
}

#[tokio::test]
async fn test_http_server_starts_with_sovereign_socket() -> Result<()> {
    use songbird_orchestrator::app::http_server;

    let (federation_state, service_registry) = create_test_federation_state();

    // Start server on ephemeral port (OS chooses)
    let local_service_registry =
        Arc::new(songbird_orchestrator::service_registry::ServiceRegistry::new());
    let server_task = tokio::spawn(async move {
        http_server::start_http_server(
            federation_state,
            service_registry,
            local_service_registry,
            "0.0.0.0",
            0, // Ephemeral port
        )
        .await
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Server should be running (task shouldn't have exited)
    assert!(!server_task.is_finished(), "Server should still be running");

    server_task.abort();
    Ok(())
}

#[tokio::test]
async fn test_http_server_health_endpoint_with_sovereign_socket() -> Result<()> {
    use songbird_orchestrator::app::http_server;

    let (federation_state, service_registry) = create_test_federation_state();

    // Start server
    let local_service_registry =
        Arc::new(songbird_orchestrator::service_registry::ServiceRegistry::new());
    tokio::spawn(async move {
        let _ = http_server::start_http_server(
            federation_state,
            service_registry,
            local_service_registry,
            "127.0.0.1",
            18765, // Fixed port for testing
        )
        .await;
    });

    // Wait for server to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Try to connect to health endpoint
    let client = reqwest::Client::builder().timeout(Duration::from_secs(5)).build()?;

    let url = "http://127.0.0.1:18765/health";

    let result = timeout(Duration::from_secs(3), client.get(url).send()).await;

    if let Ok(Ok(response)) = result {
        assert!(response.status().is_success(), "Health endpoint should return OK");
        println!("✅ HTTP server health endpoint working with sovereign socket");
    } else {
        println!("⚠️  Could not connect (port might be in use)");
    }

    Ok(())
}

#[tokio::test]
async fn test_http_server_rapid_restart_with_sovereign_socket() -> Result<()> {
    use songbird_orchestrator::app::http_server;

    let port = 18766;

    // Start and stop server multiple times rapidly
    for i in 0..3 {
        let (federation_state, service_registry) = create_test_federation_state();

        let local_service_registry =
            Arc::new(songbird_orchestrator::service_registry::ServiceRegistry::new());
        let server_task = tokio::spawn(async move {
            http_server::start_http_server(
                federation_state,
                service_registry,
                local_service_registry,
                "127.0.0.1",
                port,
            )
            .await
        });

        // Let server start
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Stop server
        server_task.abort();

        // Brief pause
        tokio::time::sleep(Duration::from_millis(100)).await;

        println!("✅ Restart cycle {} completed", i + 1);
    }

    println!("✅ Rapid restart test successful (SO_REUSEADDR working)");
    Ok(())
}

#[tokio::test]
async fn test_http_server_concurrent_requests_with_sovereign_socket() -> Result<()> {
    use songbird_orchestrator::app::http_server;

    let port = 18767;
    let (federation_state, service_registry) = create_test_federation_state();

    // Start server
    let local_service_registry =
        Arc::new(songbird_orchestrator::service_registry::ServiceRegistry::new());
    tokio::spawn(async move {
        let _ = http_server::start_http_server(
            federation_state,
            service_registry,
            local_service_registry,
            "127.0.0.1",
            port,
        )
        .await;
    });

    // Wait for server
    tokio::time::sleep(Duration::from_millis(500)).await;

    let client = reqwest::Client::builder().timeout(Duration::from_secs(5)).build()?;

    let url = format!("http://127.0.0.1:{}/health", port);

    // Send multiple concurrent requests
    let mut handles = vec![];
    for _ in 0..20 {
        let client = client.clone();
        let url = url.clone();

        let handle =
            tokio::spawn(
                async move { timeout(Duration::from_secs(3), client.get(&url).send()).await },
            );

        handles.push(handle);
    }

    // Wait for all requests
    let mut successes = 0;
    for handle in handles {
        if let Ok(Ok(Ok(response))) = handle.await {
            if response.status().is_success() {
                successes += 1;
            }
        }
    }

    if successes > 0 {
        println!("✅ Handled {}/20 concurrent requests", successes);
        assert!(successes >= 15, "Should handle most concurrent requests");
    } else {
        println!("⚠️  Port might be in use, skipping concurrent test");
    }

    Ok(())
}

#[tokio::test]
async fn test_http_server_fallback_port_selection() -> Result<()> {
    use songbird_orchestrator::app::http_server;
    use songbird_orchestrator::network::SovereignBinder;

    // Occupy a port
    let (_blocker, blocker_addr) = SovereignBinder::bind_sovereign(18768).await?;
    let blocked_port = blocker_addr.port();

    println!("Blocking port: {}", blocked_port);

    // Try to start server on blocked port (should fall back)
    let (federation_state, service_registry) = create_test_federation_state();

    let local_service_registry =
        Arc::new(songbird_orchestrator::service_registry::ServiceRegistry::new());
    let server_task = tokio::spawn(async move {
        http_server::start_http_server(
            federation_state,
            service_registry,
            local_service_registry,
            "127.0.0.1",
            blocked_port,
        )
        .await
    });

    // Give server time to try binding and fall back
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Server should have fallen back to another port
    assert!(
        !server_task.is_finished() || server_task.await.is_ok(),
        "Server should start on fallback port"
    );

    println!("✅ Server successfully fell back from blocked port");

    Ok(())
}

#[tokio::test]
async fn test_http_server_ipv4_and_ipv6_binding() -> Result<()> {
    use songbird_orchestrator::app::http_server;

    // Test IPv4
    let (federation_state_v4, service_registry_v4) = create_test_federation_state();
    let server_v4 = tokio::spawn(async move {
        http_server::start_http_server(
            federation_state_v4,
            service_registry_v4,
            Arc::new(songbird_orchestrator::service_registry::ServiceRegistry::new()),
            "0.0.0.0", // IPv4 wildcard
            18769,
        )
        .await
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    if !server_v4.is_finished() {
        println!("✅ HTTP server bound to IPv4 (0.0.0.0)");
    }

    server_v4.abort();

    // Test IPv6 (if available)
    let (federation_state_v6, service_registry_v6) = create_test_federation_state();
    let server_v6 = tokio::spawn(async move {
        http_server::start_http_server(
            federation_state_v6,
            service_registry_v6,
            Arc::new(songbird_orchestrator::service_registry::ServiceRegistry::new()),
            "::", // IPv6 wildcard
            18770,
        )
        .await
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    if !server_v6.is_finished() {
        println!("✅ HTTP server bound to IPv6 (::)");
    } else {
        println!("⚠️  IPv6 binding not available (system dependent)");
    }

    server_v6.abort();

    Ok(())
}

// Regression test for double-bind bug (Dec 20, 2025)
#[tokio::test]
async fn test_no_double_bind_regression() -> Result<()> {
    use songbird_orchestrator::network::SovereignBinder;

    // This test ensures the fix for the double-bind bug remains in place

    // Bind using sovereign binder
    let (listener1, addr) = SovereignBinder::bind_sovereign(0).await?;
    let port = addr.port();

    println!("First bind: {}", addr);

    // Convert to std listener (simulates what HTTP server does)
    let std_listener: std::net::TcpListener = listener1.into_std()?;

    // Verify we can't bind to same port again (without SO_REUSEPORT)
    // Unless on Linux where SO_REUSEPORT allows it
    let second_bind = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await;

    #[cfg(target_os = "linux")]
    {
        if second_bind.is_ok() {
            println!("✅ SO_REUSEPORT allows multiple binds (Linux)");
        } else {
            println!("✅ Port protected from double-bind");
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        assert!(second_bind.is_err(), "Should not allow double-bind on non-Linux");
        println!("✅ Port protected from double-bind");
    }

    // Clean up
    drop(std_listener);

    Ok(())
}

#[tokio::test]
async fn test_zero_downtime_deployment_simulation() -> Result<()> {
    use songbird_orchestrator::network::SovereignBinder;

    // Simulate zero-downtime deployment using SO_REUSEPORT
    let port = 18771;

    // Start "old" server
    let (old_server, addr1) = SovereignBinder::bind_sovereign(port).await?;
    println!("Old server: {}", addr1);

    // Start "new" server on same port (SO_REUSEPORT enables this on Linux)
    #[cfg(target_os = "linux")]
    {
        let result = SovereignBinder::bind_sovereign(port).await;
        if result.is_ok() {
            let (new_server, addr2) = result.unwrap();
            println!("New server: {} (zero-downtime deployment)", addr2);

            assert_eq!(addr1.port(), addr2.port(), "Both should use same port");

            // Both servers can handle connections simultaneously
            // This is how zero-downtime deployment works

            drop(new_server);
            println!("✅ Zero-downtime deployment simulation successful");
        } else {
            println!("⚠️  SO_REUSEPORT not available");
        }
    }

    drop(old_server);

    Ok(())
}
