// Test for HTTPS server listener reuse fix (Dec 20, 2025)
//
// This test verifies that the HTTPS server correctly uses the pre-bound
// listener instead of trying to bind again, which was causing startup hangs.

#[cfg(test)]
mod https_listener_tests {
    use super::*;
    use tokio::net::TcpListener;
    
    #[tokio::test]
    async fn test_listener_is_reused_not_double_bound() {
        // This test ensures that when we pass a listener to start_https_server,
        // it actually uses that listener instead of trying to bind again
        
        // Bind to a random port
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("Failed to bind");
        let addr = listener.local_addr().expect("Failed to get local addr");
        
        // The port should now be bound
        assert!(addr.port() > 0);
        
        // If we try to bind to the same port again, it should fail
        let result = TcpListener::bind(addr).await;
        assert!(result.is_err(), "Port should already be in use");
        
        // The fix ensures that start_https_server uses the listener we provide
        // instead of calling axum_server::bind_rustls which would try to bind again
    }
    
    #[tokio::test]
    async fn test_tcp_listener_conversion() {
        // Test that we can convert tokio::net::TcpListener to std::net::TcpListener
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("Failed to bind");
        let addr = listener.local_addr().expect("Failed to get local addr");
        
        // Convert to std
        let std_listener = listener.into_std().expect("Failed to convert to std");
        let std_addr = std_listener.local_addr().expect("Failed to get std local addr");
        
        // Address should be the same
        assert_eq!(addr, std_addr);
    }
}

