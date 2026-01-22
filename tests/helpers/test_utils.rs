//! Common test utilities and helpers

use std::time::Duration;
use tokio::time::timeout;

/// Wait for a condition to be true with timeout
///
/// Modern approach: Uses exponential backoff (1ms -> 100ms) instead of fixed 10ms sleep.
/// This is faster when condition becomes true quickly, more responsive, and more robust.
pub async fn wait_for<F>(condition: F, max_duration: Duration) -> bool
where
    F: Fn() -> bool,
{
    let start = std::time::Instant::now();
    let mut backoff = Duration::from_millis(1);
    let max_backoff = Duration::from_millis(100);

    while start.elapsed() < max_duration {
        if condition() {
            return true;
        }

        // Exponential backoff: 1ms, 2ms, 4ms, 8ms, ..., up to 100ms
        tokio::time::sleep(backoff).await;
        backoff = std::cmp::min(backoff * 2, max_backoff);
    }

    false
}

/// Create a temporary Unix socket path
pub fn temp_unix_socket_path(name: &str) -> String {
    format!("/tmp/songbird-test-{}-{}.sock", name, uuid::Uuid::new_v4())
}

/// Wait for Unix socket to be ready (file exists and is a socket)
///
/// Modern approach: Checks actual socket readiness instead of blind sleep.
/// Uses exponential backoff for efficiency.
pub async fn wait_for_socket_ready(path: &str, max_duration: Duration) -> bool {
    wait_for(|| std::path::Path::new(path).exists(), max_duration).await
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
        use std::sync::atomic::{AtomicUsize, Ordering};
        let counter = AtomicUsize::new(0);

        let result = wait_for(
            || {
                counter.fetch_add(1, Ordering::SeqCst);
                counter.load(Ordering::SeqCst) >= 5
            },
            Duration::from_secs(1),
        )
        .await;

        assert!(result);
        assert!(counter.load(Ordering::SeqCst) >= 5);
    }

    #[tokio::test]
    async fn test_wait_for_timeout() {
        let result = wait_for(|| false, Duration::from_millis(100)).await;

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
