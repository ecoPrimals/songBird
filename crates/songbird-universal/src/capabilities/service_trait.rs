//! Service trait for registration with the capability adapter

/// Trait for service-like types that can be registered
///
/// This trait allows different service representations (test fixtures, real services)
/// to be registered with the adapter.
pub trait ServiceLike {
    /// Get the service ID
    fn id(&self) -> &str;

    /// Get the service name
    fn name(&self) -> &str;

    /// Get the service endpoint
    fn endpoint(&self) -> &str;

    /// Get the service capabilities
    fn capabilities(&self) -> &[String];
}

// Implementation for TestService (dev-dependency, test builds only)
#[cfg(test)]
impl ServiceLike for songbird_test_utils::fixtures::TestService {
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
