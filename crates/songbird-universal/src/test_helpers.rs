//! Test helpers for E2E and integration tests
//!
//! This module provides helpers for using the registration API with test fixtures.

use crate::capabilities::{
    CapabilityError, RegistrationHandle, ServiceLike, UniversalCapabilityAdapter,
};

/// Register a service using a test fixture
///
/// This is a helper function for E2E and integration tests that need to register
/// test services with the adapter. It accepts any type that implements the required
/// methods (id, name, endpoint, capabilities).
///
/// # Examples
///
/// ```rust,ignore
/// use songbird_test_utils::fixtures::compute_service;
/// use songbird_universal::test_helpers::register_test_service;
///
/// let adapter = UniversalCapabilityAdapter::new(config);
/// let service = compute_service("test-compute");
/// let handle = register_test_service(&adapter, service).await?;
/// ```
pub async fn register_test_service<S>(
    adapter: &UniversalCapabilityAdapter,
    service: S,
) -> Result<RegistrationHandle, CapabilityError>
where
    S: TestServiceLike,
{
    adapter.register_service(TestServiceWrapper(service)).await
}

/// Trait for test service fixtures
///
/// This trait is automatically implemented for types with the expected methods.
pub trait TestServiceLike {
    /// Get service ID
    fn id(&self) -> &str;
    /// Get service name
    fn name(&self) -> &str;
    /// Get service endpoint
    fn endpoint(&self) -> &str;
    /// Get service capabilities
    fn capabilities(&self) -> &[String];
}

/// Wrapper that implements ServiceLike for test services
struct TestServiceWrapper<T>(T);

impl<T: TestServiceLike> ServiceLike for TestServiceWrapper<T> {
    fn id(&self) -> &str {
        self.0.id()
    }

    fn name(&self) -> &str {
        self.0.name()
    }

    fn endpoint(&self) -> &str {
        self.0.endpoint()
    }

    fn capabilities(&self) -> &[String] {
        self.0.capabilities()
    }
}

// Blanket implementation for TestService from songbird-test-utils
impl TestServiceLike for songbird_test_utils::fixtures::TestService {
    fn id(&self) -> &str {
        self.id()
    }

    fn name(&self) -> &str {
        self.name()
    }

    fn endpoint(&self) -> &str {
        self.endpoint()
    }

    fn capabilities(&self) -> &[String] {
        self.capabilities()
    }
}
