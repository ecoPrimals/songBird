//! Unit tests for canonical response types

#[cfg(test)]
mod tests {
    #![allow(clippy::uninlined_format_args)]
    #![allow(clippy::float_cmp)]
    #![allow(clippy::useless_vec)]
    #![allow(clippy::unreadable_literal)]

    use crate::responses::*;

    #[test]
    fn test_songbird_response_success() {
        let response: SongbirdResponse<String> = SongbirdResponse::success("test data".to_string());
        assert_eq!(response.data, "test data");
    }

    #[test]
    fn test_songbird_response_with_confidence() {
        let response: SongbirdResponse<i32> = SongbirdResponse::success(42).with_confidence(0.8);
        assert_eq!(response.data, 42);
    }

    #[test]
    fn test_songbird_response_with_human_context() {
        let response: SongbirdResponse<String> =
            SongbirdResponse::success("data".to_string()).with_human_context("This is a test");
        assert!(response.human_context.is_some());
        assert_eq!(response.human_context.unwrap(), "This is a test");
    }

    #[test]
    fn test_songbird_response_clone() {
        let response1: SongbirdResponse<String> = SongbirdResponse::success("test".to_string());
        let response2 = response1.clone();
        assert_eq!(response1.data, response2.data);
    }

    #[test]
    fn test_response_performance_default() {
        let perf = ResponsePerformance::default();
        // processing_time_ms is u64, check it's initialized
        // processing_time_ms is u64, always >= 0
        assert!(perf.memory_usage_bytes.is_none());
    }
}
