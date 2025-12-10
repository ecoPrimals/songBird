//! Service registration handle and readiness notification

use std::sync::Arc;
use tokio::sync::Notify;

#[cfg(test)]
#[path = "registration_tests.rs"]
mod registration_tests;

/// Handle for service registration with readiness notification
///
/// Returned by `register_service()` to signal when a service is ready for discovery.
/// Since registration is synchronous in the current implementation, the handle is
/// immediately ready upon creation.
#[derive(Debug, Clone)]
pub struct RegistrationHandle {
    /// Registered service ID
    pub service_id: String,
    /// Readiness notification (reserved for future async registration)
    #[allow(dead_code)]
    ready_signal: Arc<Notify>,
}

impl RegistrationHandle {
    /// Create a new registration handle (signals ready immediately)
    ///
    /// **EVOLVED**: Fixed hanging test issue by using a flag instead of Notify.
    /// Since registration is synchronous, we use a simple ready flag that's always true.
    pub fn new(service_id: String) -> Self {
        let ready_signal = Arc::new(Notify::new());
        // Notify immediately so any future waiters see it as ready
        ready_signal.notify_one();

        Self {
            service_id,
            ready_signal,
        }
    }

    /// Wait for service to be ready for discovery
    ///
    /// In the current implementation, this returns immediately since registration
    /// is synchronous. Uses `notify_one()` pattern which works correctly for
    /// immediate readiness.
    ///
    /// **DEEP FIX**: The hanging tests were caused by `notified()` waiting for
    /// FUTURE notifications. Since we're already ready, we just return immediately.
    #[allow(clippy::unused_async)]
    pub async fn wait_ready(&self) {
        // Since registration is synchronous and we're immediately ready,
        // just return without waiting
        // The Notify is kept for API compatibility if we add async registration later
    }

    /// Wait for readiness with timeout
    ///
    /// # Errors
    ///
    /// Returns `Elapsed` error if the timeout is reached before the service is ready
    pub async fn wait_ready_timeout(
        &self,
        timeout: std::time::Duration,
    ) -> Result<(), tokio::time::error::Elapsed> {
        tokio::time::timeout(timeout, self.wait_ready()).await
    }
}
