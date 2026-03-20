// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Unit tests for canonical response types

#[cfg(test)]
mod tests {
    #![allow(clippy::uninlined_format_args)]
    #![allow(clippy::float_cmp)]
    #![allow(clippy::useless_vec)]
    #![allow(clippy::unreadable_literal)]

    use crate::SuggestedAction;
    use crate::metadata::AIResponseMetadata;
    use crate::responses::*;
    use std::time::Instant;

    #[test]
    fn test_songbird_response_success() {
        let response: SongbirdResult<String> = SongbirdResult::success("test data".to_string());
        assert_eq!(response.data, "test data");
    }

    #[test]
    fn test_songbird_response_with_confidence() {
        let response: SongbirdResult<i32> = SongbirdResult::success(42).with_confidence(0.8);
        assert_eq!(response.data, 42);
    }

    #[test]
    fn test_songbird_response_with_human_context() {
        let response: SongbirdResult<String> =
            SongbirdResult::success("data".to_string()).with_human_context("This is a test");
        assert!(response.human_context.is_some());
        assert_eq!(response.human_context.unwrap(), "This is a test");
    }

    #[test]
    fn test_songbird_response_clone() {
        let response1: SongbirdResult<String> = SongbirdResult::success("test".to_string());
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

    #[test]
    fn test_songbird_response_with_suggestion() {
        let action = SuggestedAction::new("test_action", "A test action");
        let response: SongbirdResult<i32> = SongbirdResult::success(42).with_suggestion(action);
        assert_eq!(response.suggested_actions.len(), 1);
    }

    #[test]
    fn test_songbird_response_with_ai_metadata() {
        let metadata = AIResponseMetadata::default();
        let response: SongbirdResult<i32> = SongbirdResult::success(42).with_ai_metadata(metadata);
        // Just verify it compiles and runs - metadata is opaque
        assert_eq!(response.data, 42);
    }

    #[test]
    fn test_songbird_response_finish_processing() {
        let start = Instant::now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let response: SongbirdResult<i32> = SongbirdResult::success(42).finish_processing(start);
        assert!(response.performance.processing_time_ms >= 10);
    }

    #[test]
    fn test_songbird_response_map() {
        let response: SongbirdResult<i32> = SongbirdResult::success(42);
        let mapped: SongbirdResult<String> = response.map(|x| format!("value: {x}"));
        assert_eq!(mapped.data, "value: 42");
    }

    #[test]
    fn test_songbird_response_into_data() {
        let response: SongbirdResult<String> = SongbirdResult::success("test".to_string());
        let data = response.into_data();
        assert_eq!(data, "test");
    }

    #[test]
    fn test_songbird_response_data_ref() {
        let response: SongbirdResult<String> = SongbirdResult::success("test".to_string());
        assert_eq!(response.data(), "test");
    }

    #[test]
    fn test_songbird_response_from() {
        let response: SongbirdResult<i32> = 42.into();
        assert_eq!(response.data, 42);
    }

    #[test]
    fn test_songbird_result_unit() {
        let response = SongbirdResult::unit();
        assert_eq!(response.data, ());
    }

    #[test]
    fn test_cache_status_variants() {
        assert_eq!(CacheStatus::Hit, CacheStatus::Hit);
        assert_eq!(CacheStatus::Miss, CacheStatus::Miss);
        assert_eq!(CacheStatus::NotApplicable, CacheStatus::NotApplicable);
        assert_eq!(CacheStatus::Bypassed, CacheStatus::Bypassed);
        assert_ne!(CacheStatus::Hit, CacheStatus::Miss);
    }

    #[test]
    fn test_response_performance_with_values() {
        let perf = ResponsePerformance {
            processing_time_ms: 100,
            memory_usage_bytes: Some(1024),
            cpu_usage_percent: Some(50.0),
            network_rtt_ms: Some(20),
            cache_status: CacheStatus::Hit,
        };
        assert_eq!(perf.processing_time_ms, 100);
        assert_eq!(perf.memory_usage_bytes, Some(1024));
        assert_eq!(perf.cpu_usage_percent, Some(50.0));
        assert_eq!(perf.network_rtt_ms, Some(20));
        assert_eq!(perf.cache_status, CacheStatus::Hit);
    }
}
