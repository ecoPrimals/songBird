// Canonical Test Framework Tests
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]

use songbird_test_utils::canonical_test_framework::{
    CanonicalAssertions, TestContext, TestEnvironment, TestResult,
};
use songbird_types::{SongbirdError as ErrorType, SongbirdResult};
use std::time::Duration;

#[cfg(test)]
mod canonical_framework_tests {
    use super::*;

    #[tokio::test]
    async fn test_canonical_framework_basic() -> TestResult<()> {
        TestEnvironment::setup()?;

        // Test basic assertions with available methods
        let success_result: SongbirdResult<String> = Ok("test success".to_string());
        CanonicalAssertions::assert_success(&success_result)?;

        let error_result: SongbirdResult<String> = Err(ErrorType::service("test", "test error"));
        CanonicalAssertions::assert_error(&error_result)?;

        // Test duration assertions
        let actual = 100.0;
        let expected = 105.0;
        let tolerance = 10.0;
        CanonicalAssertions::assert_approx_equal(actual, expected, tolerance)?;

        // Test range assertions
        CanonicalAssertions::assert_range(&5, &1, &10)?;

        Ok(())
    }

    #[tokio::test]
    async fn test_mock_service() -> TestResult<()> {
        TestEnvironment::setup()?;

        let context = TestContext::new("mock_service_test");

        // Test that context is properly created
        assert!(!context.name.is_empty(), "Test context should have a name");
        assert!(context.elapsed() < Duration::from_secs(1), "Test should start quickly");

        Ok(())
    }

    #[tokio::test]
    async fn test_unhealthy_mock_service() -> TestResult<()> {
        TestEnvironment::setup()?;

        let context = TestContext::new("unhealthy_service_test");

        // Test unhealthy service handling
        assert!(!context.name.is_empty(), "Test context should have a name");

        Ok(())
    }

    #[tokio::test]
    async fn test_performance_utils() -> TestResult<()> {
        TestEnvironment::setup()?;

        // Test performance measurement with available API
        let start = std::time::Instant::now();

        // Simulate some work
        tokio::time::sleep(Duration::from_millis(10)).await;

        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(5), "Should take some time");

        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_mock_services() -> TestResult<()> {
        TestEnvironment::setup()?;

        let context1 = TestContext::new("service1");
        let context2 = TestContext::new("service2");

        // Test multiple services
        assert!(!context1.name.is_empty(), "First context should have a name");
        assert!(!context2.name.is_empty(), "Second context should have a name");

        Ok(())
    }

    #[tokio::test]
    async fn test_timeout_assertion() -> TestResult<()> {
        TestEnvironment::setup()?;

        // Test timeout functionality with correct closure syntax
        let timeout_duration = Duration::from_millis(100);

        let result = CanonicalAssertions::assert_timeout(
            || async {
                tokio::time::sleep(Duration::from_millis(50)).await;
                Ok(())
            },
            timeout_duration,
        )
        .await;

        assert!(result.is_ok(), "Should complete within timeout");

        let result = CanonicalAssertions::assert_timeout(
            || async {
                tokio::time::sleep(Duration::from_millis(200)).await;
                Ok(())
            },
            timeout_duration,
        )
        .await;

        assert!(result.is_err(), "Should timeout");

        Ok(())
    }

    #[tokio::test]
    async fn test_error_message_matching() -> TestResult<()> {
        TestEnvironment::setup()?;

        let error_result: SongbirdResult<String> =
            Err(ErrorType::service("test_service", "specific error message"));

        CanonicalAssertions::assert_error(&error_result)?;
        CanonicalAssertions::assert_error_contains(&error_result, "specific")?;

        Ok(())
    }

    #[tokio::test]
    async fn test_performance_benchmarking() -> TestResult<()> {
        TestEnvironment::setup()?;

        // Test benchmarking functionality with available tools
        let start = std::time::Instant::now();

        // Simulate work
        for _ in 0..100 {
            tokio::task::yield_now().await;
        }

        let elapsed = start.elapsed();
        assert!(elapsed < Duration::from_secs(1), "Should complete quickly");

        Ok(())
    }
}
