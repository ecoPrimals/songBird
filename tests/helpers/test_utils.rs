//! Common test utilities and helpers

use std::time::Duration;
use tokio::time::timeout;

/// Wait for a condition to be true with timeout
pub async fn wait_for<F>(condition: F, max_duration: Duration) -> bool
where
    F: Fn() -> bool,
{
    let start = std::time::Instant::now();
    
    while start.elapsed() < max_duration {
        if condition() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    false
}

/// Create a temporary Unix socket path
pub fn temp_unix_socket_path(name: &str) -> String {
    format!("/tmp/songbird-test-{}-{}.sock", name, uuid::Uuid::new_v4())
}

/// Clean up Unix socket file
pub fn cleanup_socket(path: &str) {
    let _ = std::fs::remove_file(path);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_wait_for_success() {
        let mut counter = 0;
        
        let result = wait_for(
            || {
                counter += 1;
                counter >= 5
            },
            Duration::from_secs(1),
        ).await;
        
        assert!(result);
        assert!(counter >= 5);
    }
    
    #[tokio::test]
    async fn test_wait_for_timeout() {
        let result = wait_for(
            || false,
            Duration::from_millis(100),
        ).await;
        
        assert!(!result);
    }
    
    #[test]
    fn test_temp_unix_socket_path() {
        let path1 = temp_unix_socket_path("test");
        let path2 = temp_unix_socket_path("test");
        
        assert!(path1.starts_with("/tmp/songbird-test-test-"));
        assert!(path2.starts_with("/tmp/songbird-test-test-"));
        assert_ne!(path1, path2); // Should be unique
    }
}

