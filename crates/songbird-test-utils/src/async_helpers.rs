// Async test helpers for songbird testing

use songbird_errors::{SongbirdError, Result as SongbirdResult};
use std::future::Future;
use std::time::Duration;
use tokio::time::{sleep, timeout};

/// Default timeout for async operations in tests
// MOVED: Use songbird_config::constants::testing::testing::ASYNC_TEST_TIMEOUT
/// Default delay for async operations that need time to propagate
// MOVED: Use songbird_config::constants::testing::ASYNC_DELAY
/// Create a test timeout with custom duration
pub async fn test_timeout<T>(
    future: impl Future<Output = T>,
    duration: Duration,
) -> Result<T, SongbirdError> {
    timeout(duration, future)
        .await
        .map_err(|_| SongbirdError::operation_error("Test timeout exceeded"))
}

/// Wait for a condition to become true with polling
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

    Err(SongbirdError::internal_error(format!(
        "Condition not met within {max_wait:?}"
    )))
}

/// Retry an operation with exponential backoff
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
                    return Err(SongbirdError::internal_error(format!(
                        "Operation failed after {max_retries} retries: {e}"
                    )));
                }
                sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
        }
    }

    unreachable!("Loop should have returned or errored")
}
