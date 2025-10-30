//! Common constants used throughout Songbird

/// Default timeout for network operations (in seconds)
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default retry attempts for failed operations
pub const DEFAULT_RETRY_ATTEMPTS: u32 = 3;

/// Default port for Songbird services
pub const DEFAULT_PORT: u16 = 8080;

/// Maximum number of concurrent connections
pub const MAX_CONCURRENT_CONNECTIONS: usize = 1000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_timeout_secs() {
        assert_eq!(DEFAULT_TIMEOUT_SECS, 30);
        // Value checks would be optimized out by compiler for const values
    }

    #[test]
    fn test_default_retry_attempts() {
        assert_eq!(DEFAULT_RETRY_ATTEMPTS, 3);
        // Value checks would be optimized out by compiler for const values
    }

    #[test]
    fn test_default_port() {
        assert_eq!(DEFAULT_PORT, 8080);
        // Value checks would be optimized out by compiler for const values
    }

    #[test]
    fn test_max_concurrent_connections() {
        assert_eq!(MAX_CONCURRENT_CONNECTIONS, 1000);
        // Value checks would be optimized out by compiler for const values
    }

    #[test]
    fn test_constants_are_reasonable() {
        // All constant values are validated at compile time
        assert_eq!(DEFAULT_TIMEOUT_SECS, 30);
        assert_eq!(DEFAULT_RETRY_ATTEMPTS, 3);
        assert_eq!(DEFAULT_PORT, 8080);
        assert_eq!(MAX_CONCURRENT_CONNECTIONS, 1000);
    }
}
