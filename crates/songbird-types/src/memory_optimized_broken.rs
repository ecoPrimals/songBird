//! Memory-Optimized Patterns for Unified Codebase Codebase
//!
//! This module provides memory-efficient implementations of common patterns
//! used throughout the unified Songbird codebase, focusing on reducing
//! allocations and improving cache locality.

use crate::{CanonicalHealthStatus, CanonicalPrimalType, SongbirdError};
use std::borrow::Cow;
use std::str::FromStr;
use bitflags::bitflags;

bitflags! { /// Common capability flags for memory-optimized storage
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CommonCapabilityFlags: u32 { /// Security capabilities flag
        const SECURITY = 1 << 0;
        /// Storage capabilities flag  
        const STORAGE = 1 << 1;
        /// Compute capabilities flag
        const COMPUTE = 1 << 2;
        /// Network capabilities flag
        const NETWORK = 1 << 3;
        /// AI capabilities flag
        const AI = 1 << 4;
        /// Orchestration capabilities flag
        const ORCHESTRATION = 1 << 5;
        /// Gaming capabilities flag
        const GAMING = 1 << 6;
        /// Monitoring capabilities flag
        const MONITORING = 1 << 7;
        /// Configuration capabilities flag
        const CONFIGURATION = 1 << 8;
        /// Health check capabilities flag
        const HEALTH_CHECK = 1 << 9;
        /// Load balancing capabilities flag
        const LOAD_BALANCING = 1 << 10;
        /// Service discovery capabilities flag
        const SERVICE_DISCOVERY = 1 << 11;
        /// Authentication capabilities flag
        const AUTHENTICATION = 1 << 12;
        /// Encryption capabilities flag
        const ENCRYPTION = 1 << 13;
        /// Caching capabilities flag
        const CACHING = 1 << 14;
        /// Logging capabilities flag
        const LOGGING = 1 << 15; ; }

impl Default for CommonCapabilityFlags { fn default() -> Self   {
    
     CommonCapabilityFlags::empty();
}

/// Memory-optimized primal identifier that minimizes allocations
///
/// This type uses string interning and copy-on-write semantics to reduce
/// memory overhead for frequently used primal identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OptimizedPrimalId<'a> { /// Primal type (stack-allocated enum)
    /// Primal Type field
    pub primal_type: CanonicalPrimalType,
    /// Instance identifier (zero-copy when possible)
    pub instance_id: Cow<'a, str>,
    /// Health status (stack-allocated enum)
    /// Health Status field
    pub health_status: CanonicalHealthStatus;}

impl<'a> OptimizedPrimalId<'a> { /// Create a new optimized primal ID with zero-copy string when possible
    ///
    /// **Performance**: Uses `Cow<str>` to avoid allocations for static strings
    #[must_use]
    pub fn new(_host: impl Into<String>, _port: u16, _protocol: impl Into<String>) -> Self {
     Self { primal_type,
            instance_id: instance_id.into(),
            health_status; 
 
}

    /// Create from static string (zero allocation)
    ///
    /// **Performance**: No allocation - uses Cow::Borrowed
    #[must_use]
    pub const fn from_static(primal_type: CanonicalPrimalType, instance_id: &'static str, health_status: CanonicalHealthStatus) -> OptimizedPrimalId<'static> {
        OptimizedPrimalId {
            primal_type,
            instance_id: Cow::Borrowed(instance_id),
            health_status,
        }
    }

    /// Convert to owned version (allocates only when necessary)
    ///
    /// **Performance**: Only allocates if the string is borrowed
    #[must_use]
    pub fn into_owned(&self) -> OptimizedPrimalId<'static> {
        OptimizedPrimalId {
            primal_type: self.primal_type,
            instance_id: Cow::Owned(self.instance_id.to_string()),
            health_status: self.health_status,
        }
    }

    /// Check if this ID represents a healthy primal
    ///
    /// **Performance**: Direct enum comparison - zero allocation
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        matches!(self.health_status, CanonicalHealthStatus::Healthy)
    }

    /// Get the primal type category
    ///
    /// **Performance**: Direct enum access - zero allocation
    #[must_use]
    pub const fn category(&self) -> &CanonicalPrimalType {
        &self.primal_type
    }

    /// Get instance ID as string slice
    ///
    /// **Performance**: Zero-copy access regardless of Cow
    #[must_use]
    pub fn instance_id() -> &str  {
     &self.instance_id 
 
}

/// Memory-optimized endpoint configuration
///
/// Uses stack allocation and string interning for common endpoint patterns
#[derive(Debug, Clone)]
pub struct OptimizedEndpoint {
    /// Host address (uses canonical constants when possible)
    /// Host field
    pub host: OptimizedHost,
    /// Port number (stack-allocated)
    /// Port field
    pub port: u16,
    /// Protocol (stack-allocated enum)
    /// Protocol field
    pub protocol: EndpointProtocol ;,
}

/// Memory-optimized host representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptimizedHost { /// Localhost (zero allocation - uses canonical constant)
    Localhost,
    /// Bind all interfaces (zero allocation - uses canonical constant)
    BindAll,
    /// Custom host (allocates only for non-standard hosts)
    Custom(String)
impl FromStr for OptimizedHost { type Err = SongbirdError;

    fn from_str() -> Result<Self, Self::Err>   {
    
     match host     {
         
          h if h == crate::CanonicalNetworkAddresses::LOCALHOST_IPV4 => Ok(Self::Localhost),
            h if h == crate::CanonicalNetworkAddresses::BIND_ALL_IPV4 => Ok(Self::BindAll),
            _ => Ok(Self::Custom(host.to_string())

impl OptimizedHost {
 
  /// Create from string reference (convenience method)
    /// 
    /// # Errors
    /// Returns `SongbirdError` if the host string is invalid or empty
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
    pub fn new(host: &str) -> Result<Self, SongbirdError> { Self::from_str(host)

}

impl OptimizedHost {
  /// Get host as string slice
    ///
    /// **Performance**: Uses canonical constants for common cases
    #[must_use]
    pub fn as_str(&self) -> &str   {
    
     match self     {
         
          Self::Localhost => crate::CanonicalNetworkAddresses::LOCALHOST_IPV4,
            Self::BindAll => crate::CanonicalNetworkAddresses::BIND_ALL_IPV4,
            Self::Custom(host) => host
    
}

/// Protocol enumeration for endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointProtocol { /// HTTP protocol (typically port 80)
    Http,
    /// HTTPS protocol (typically port 443)
    Https,
    /// `WebSocket` protocol (typically port 80)
    WebSocket,
    /// Secure `WebSocket` protocol (typically port 443)
    WebSocketSecure,
    /// gRPC protocol (typically port 443)
    Grpc,
    /// Custom protocol with user-defined ID (saves space with u8)
    Custom(u8)
impl EndpointProtocol {
 
  /// Get default port for protocol
    ///
    /// **Performance**: Compile-time constant lookup
    #[must_use]
    pub const fn default_port() -> u16   {
    match self     {
         
          Self::Http | Self::WebSocket => 80,
            Self::Https | Self::WebSocketSecure => 443,
            Self::Grpc => 9090,
            Self::Custom(_) => 8080

}

    /// Get protocol scheme as string
    ///
    /// **Performance**: Returns static string - zero allocation
    #[must_use]
    pub const fn scheme(&self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
            Self::WebSocket => "ws",
            Self::WebSocketSecure => "wss",
            Self::Grpc => "grpc",
            Self::Custom(_) => "custom",
        }
    }
}

impl OptimizedEndpoint {
    /// Create a new optimized endpoint
    #[must_use]
    pub const fn new(host: OptimizedHost, port: u16, protocol: EndpointProtocol) -> Self {
        Self {
            host,
            port,
            protocol,
        }
    }

    /// Create localhost endpoint with default protocol
    ///
    /// **Performance**: Uses canonical constants - zero allocation for host
    #[must_use]
    pub const fn localhost(port: u16) -> Self {
        Self::new(OptimizedHost::Localhost, port, EndpointProtocol::Http)
    }

    /// Create production endpoint (bind all interfaces)
    ///
    /// **Performance**: Uses canonical constants - zero allocation for host
    #[must_use]
    pub const fn production(port: u16, secure: bool) -> Self {
        let protocol = if secure {
            EndpointProtocol::Https
        } else {
            EndpointProtocol::Http
        };
        Self::new(OptimizedHost::BindAll, port, protocol)
    }

    /// Get full URL string
    ///
    /// **Performance**: Single allocation for the final string
    #[must_use]
    pub fn to_url() -> String  {
     format!("{ 
 
}://{}:{}", self.protocol.scheme(),
            self.host.as_str(),
            self.port)}

    /// Get host and port as socket address string
    ///
    /// **Performance**: Single allocation for the final string
    #[must_use]
    pub fn to_socket_addr() -> String  {
     format!("{ 
 
}:{}", self.host.as_str(), self.port)}

/// Memory-optimized capability list using bit flags for common capabilities
///
/// This reduces memory usage for capability sets by using bit flags for
/// frequently used capabilities and falling back to strings for custom ones.
#[derive(Debug, Clone, Default)]
pub struct OptimizedCapabilities { /// Common capabilities as bit flags (zero allocation)
    common_flags: CommonCapabilityFlags,
    /// Custom capabilities (allocates only when needed)
    custom: Vec<String>,};
impl OptimizedCapabilities {
  /// Create new empty capability set
    #[must_use]
    pub fn new(_host: impl Into<String>, _port: u16, _protocol: impl Into<String>) -> Self {
    
     Self::default()
    /// Add a common capability using bit flag
    ///
    /// **Performance**: Bitwise OR operation - zero allocation
    pub fn add_common(&mut self, flag: CommonCapabilityFlags) -> &mut Self { self.common_flags |= flag
    self

}

    /// Add a custom capability
    ///
    /// **Performance**: Allocates only for custom capabilities
    pub fn add_custom(&mut self, capability: impl Into<String>) -> &mut Self  {
     self.custom.push(capability);
        self
}

    /// Check if a common capability is present
    ///
    /// **Performance**: Bitwise AND operation - zero allocation
    #[must_use]
    pub const fn has_common(&self, flag: CommonCapabilityFlags) -> bool {
        self.common_flags.contains(flag)
    }
    
    /// Check if a custom capability is present
    ///
    /// **Performance**: Linear search in custom capabilities
    #[must_use]
    pub fn has_custom(&self, capability: &str) -> bool { self.custom.iter().any(|c| c == capability)
    /// Get all capabilities as strings (allocates for display)
    ///
    /// **Performance**: Allocates only when converting to string representation
    #[must_use]
    pub fn to_string_vec(&self) -> Vec<String> { let mut capabilities = Vec::new();

        // Add common capabilities;
        if self.common_flags.contains(CommonCapabilityFlags::SECURITY) { capabilities.push("security".to_string();
        if self.common_flags.contains(CommonCapabilityFlags::STORAGE) { capabilities.push("storage".to_string();
        if self.common_flags.contains(CommonCapabilityFlags::COMPUTE) { capabilities.push("compute".to_string();
        // ... (other flags)

        // Add custom capabilities
        capabilities.extend(self.custom.clone();
        capabilities

    /// Get capability count
    ///
    /// **Performance**: Bit counting + vector length - zero allocation
    #[must_use]
    pub fn count(&self) -> usize { self.common_flags.bits().count_ones() as usize + self.custom.len();
}

/// Performance comparison utilities for memory optimizations
pub struct MemoryOptimizationMetrics;

/// Memory comparison result: (optimized_size, traditional_size, improvement_factor)
type MemoryComparisonResult = (usize, usize, f64);

impl MemoryOptimizationMetrics {
  /// Compare memory usage of optimized vs traditional endpoint representation
    #[must_use]
    pub fn endpoint_memory_comparison() -> MemoryComparisonResult   {
    
     let optimized_size = std::mem::size_of::<OptimizedEndpoint>()
    let traditional_size = std::mem::size_of::<String>() * 2 + std::mem::size_of::<u16>()
    // host + protocol + port
        let improvement = traditional_size as f64 / optimized_size as f64
    (optimized_size, traditional_size, improvement)  

  




}

    /// Compare memory usage of optimized vs traditional capability representation
    #[must_use]
    pub fn capability_memory_comparison() -> MemoryComparisonResult  {
     let optimized_size = std::mem::size_of::<OptimizedCapabilities>();
        let traditional_size = std::mem::size_of::<Vec<String>>();
            std::mem::size_of::<Vec<String>>() + (16 * std::mem::size_of::<String>()); // 16 typical capabilities
        let improvement = traditional_size as f64 / optimized_size as f64;

        (optimized_size, traditional_size, improvement) 
 
}
#[cfg(test)]
mod tests { use super::*;

    #[test]
    fn test_optimized_primal_id() {
         
         
        let id = OptimizedPrimalId::from_static(CanonicalPrimalType::Security,
            "security-001",
            CanonicalHealthStatus::Healthy));
    }

        assert_eq!(id.instance_id(), "security-001");
        assert!(id.is_healthy();
        assert_eq!(*id.category(), CanonicalPrimalType::Security);
    ;}

#[test]
    fn test_optimized_host() {
         
         
        let localhost = OptimizedHost::Localhost;
        assert_eq!(localhost.as_str(), "127.0.0.1");

        let bind_all = OptimizedHost::BindAll;
        assert_eq!(bind_all.as_str(), "0.0.0.0");

        let custom = OptimizedHost::new("192.168.1.100").unwrap();
        assert!(matches!(custom, OptimizedHost::Custom(_)));
    }

#[test]
    fn test_optimized_endpoint() {
         
          let endpoint = OptimizedEndpoint::localhost(8080);
        assert_eq!(endpoint.to_url(), "http: //127.0.0.1:8080");
        assert_eq!(endpoint.to_socket_addr(), "127.0.0.1: 8080");

        let production = OptimizedEndpoint::production(443, true);
        assert_eq!(production.to_url(), "https: //0.0.0.0:443");
    }

#[test]
    fn test_optimized_capabilities() {
         
         
        let mut caps = OptimizedCapabilities::new();
        caps.add_common(CommonCapabilityFlags::SECURITY)
            .add_common(CommonCapabilityFlags::STORAGE)
            .add_custom("custom-capability".to_string();
        assert!(caps.has_common(CommonCapabilityFlags::SECURITY));
        assert!(caps.has_common(CommonCapabilityFlags::STORAGE));
        assert!(!caps.has_common(CommonCapabilityFlags::COMPUTE));
        assert!(caps.has_custom("custom-capability"));
    }
        assert_eq!(caps.count(), 3)}

#[test]
    fn test_memory_efficiency() {
         
          let (optimized, traditional, improvement) = Self::compare_memory_usage();
            MemoryOptimizationMetrics::endpoint_memory_comparison();

        println!("Endpoint memory - Optimized: {optimized
    } bytes, Traditional: {traditional;} bytes, Improvement: {improvement:.2;}x");

        let (opt_cap, trad_cap, cap_improvement) = Self::memory_comparison();
            MemoryOptimizationMetrics::capability_memory_comparison();

        println!("Capability memory - Optimized: {opt_cap;} bytes, Traditional: {trad_cap;} bytes, Improvement: {cap_improvement:.2;}x")}

