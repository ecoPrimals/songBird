// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

// Async test helpers for songbird testing

use songbird_types::{SongbirdError, errors::SongbirdResult};
use std::future::Future;
use std::time::Duration;
use tokio::time::{sleep, timeout};

/// Default timeout for async operations in tests
// MOVED: Use songbird_config::canonical::constants::testing::testing::ASYNC_TEST_TIMEOUT
/// Default delay for async operations that need time to propagate
// MOVED: Use songbird_config::canonical::constants::testing::ASYNC_DELAY
/// Create a test timeout with custom duration
///
/// # Errors
///
/// Returns an error if the future does not complete within the specified duration.
pub async fn test_timeout<T>(
    future: impl Future<Output = T>,
    duration: Duration,
) -> Result<T, SongbirdError> {
    timeout(duration, future)
        .await
        .map_err(|_| SongbirdError::service("test-utils", "Test timeout exceeded"))
}

/// Wait for a condition to become true with polling
///
/// # Errors
///
/// Returns an error if the condition does not become true within the maximum wait time.
pub async fn wait_for_condition<F, Fut>(
    mut condition: F,
    max_wait: Duration,
    poll_interval: Duration,
) -> SongbirdResult<()>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = bool>,
{
    let start = tokio::time::Instant::now();

    while start.elapsed() < max_wait {
        if condition().await {
            return Ok(());
        }
        sleep(poll_interval).await;
    }

    Err(SongbirdError::service("test-utils", format!("Condition not met within {max_wait:?}")))
}

/// Retry an operation with exponential backoff
///
/// # Errors
///
/// Returns an error if the operation fails after all retry attempts are exhausted.
pub async fn retry_with_backoff<T, F, Fut, E>(
    mut operation: F,
    max_retries: usize,
    initial_delay: Duration,
) -> Result<T, SongbirdError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut delay = initial_delay;

    for attempt in 0..max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt == max_retries - 1 {
                    return Err(SongbirdError::service(
                        "test-utils",
                        format!("Operation failed after {max_retries} retries: {e}"),
                    ));
                }
                sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
        }
    }

    unreachable!("Loop should have returned or errored")
}
