pub mod accessibility;
pub mod firewall;
pub mod security;

// Universal security integration for any primal with security capabilities  
pub mod universal_security_integration;

// Test-related modules
#[cfg(test)]
pub mod test_impls;
#[cfg(test)]
pub mod test_types;

// Re-export universal security integration only - no deprecated APIs
pub use universal_security_integration::UniversalSecurityIntegration;

// Re-export security types
pub use security::*;
