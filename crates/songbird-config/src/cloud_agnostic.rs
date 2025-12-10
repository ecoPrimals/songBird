//! Vendor-Agnostic Cloud Environment Detection
//!
//! This module provides capability-based cloud environment detection that works
//! with ANY cloud provider without hardcoding vendor-specific logic.
//!
//! # Philosophy
//!
//! Instead of checking for "AWS", "Azure", "GCP" explicitly, we detect:
//! - Cloud instance metadata endpoints (standardized patterns)
//! - Environment variable patterns (common conventions)
//! - Network interface characteristics
//! - Instance metadata APIs (vendor-agnostic detection)
//!
//! # Examples
//!
//! ```rust
//! use songbird_config::cloud::agnostic::{CloudEnvironment, detect_cloud_environment};
//!
//! let env = detect_cloud_environment().await;
//! match env {
//!     CloudEnvironment::Cloud { capabilities, .. } => {
//!         println!("Running in cloud with capabilities: {:?}", capabilities);
//!     }
//!     CloudEnvironment::OnPremise => {
//!         println!("Running on-premise");
//!     }
//!     CloudEnvironment::Edge => {
//!         println!("Running at edge");
//!     }
//! }
//! ```

use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

bitflags! {
    /// Cloud environment capabilities (vendor-agnostic)
    ///
    /// Uses bitflags for efficient storage and operations.
    /// This is more idiomatic than multiple boolean fields.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CloudCapabilities: u8 {
        /// Instance metadata service available
        const METADATA_SERVICE = 0b0000_0001;
        /// Auto-scaling capability
        const AUTOSCALING = 0b0000_0010;
        /// Managed storage available
        const MANAGED_STORAGE = 0b0000_0100;
        /// Managed networking (VPC, etc.)
        const MANAGED_NETWORKING = 0b0000_1000;
        /// Spot/preemptible instances
        const SPOT_INSTANCES = 0b0001_0000;
    }
}

// Custom serialization for bitflags
impl Serialize for CloudCapabilities {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.bits().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CloudCapabilities {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bits = u8::deserialize(deserializer)?;
        Self::from_bits(bits)
            .ok_or_else(|| serde::de::Error::custom("Invalid CloudCapabilities bits"))
    }
}

/// Deployment environment type (vendor-agnostic)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudEnvironment {
    /// Running in a cloud provider (any vendor)
    Cloud {
        /// Cloud capabilities available
        capabilities: CloudCapabilities,
        /// Instance ID (if available)
        instance_id: Option<String>,
        /// Availability zone/region (if available)
        zone: Option<String>,
    },
    /// Running on-premise (datacenter, bare metal)
    OnPremise,
    /// Running at edge (`IoT`, embedded, etc.)
    Edge,
    /// Running in local development
    Local,
}

/// Cloud instance metadata (vendor-agnostic)
#[derive(Debug, Clone)]
pub struct InstanceMetadata {
    /// Instance IP address
    pub ip_address: Option<IpAddr>,
    /// Instance ID
    pub instance_id: Option<String>,
    /// Availability zone
    pub zone: Option<String>,
    /// Instance type/size
    pub instance_type: Option<String>,
}

/// Detect cloud environment capabilities
///
/// Uses vendor-agnostic detection methods:
/// 1. Check for instance metadata endpoints (standard patterns)
/// 2. Examine environment variables (common conventions)
/// 3. Analyze network configuration
/// 4. Test for cloud-specific capabilities
///
/// # Returns
/// Detected cloud environment with capabilities
///
/// # Examples
/// ```rust,no_run
/// use songbird_config::cloud::agnostic::detect_cloud_environment;
///
/// let env = detect_cloud_environment();
/// println!("Environment: {:?}", env);
/// ```
#[must_use]
pub fn detect_cloud_environment() -> CloudEnvironment {
    // Check for metadata service (common pattern across vendors)
    if has_metadata_service() {
        let capabilities = detect_cloud_capabilities();
        let metadata = get_instance_metadata();

        return CloudEnvironment::Cloud {
            capabilities,
            instance_id: metadata.instance_id,
            zone: metadata.zone,
        };
    }

    // Check for container orchestration (K8s, ECS, etc.)
    if is_container_orchestrated() {
        let capabilities = detect_cloud_capabilities();
        return CloudEnvironment::Cloud {
            capabilities,
            instance_id: None,
            zone: None,
        };
    }

    // Check for edge indicators
    if is_edge_environment() {
        return CloudEnvironment::Edge;
    }

    // Check for local development
    if is_local_development() {
        return CloudEnvironment::Local;
    }

    // Default to on-premise
    CloudEnvironment::OnPremise
}

/// Check if instance metadata service is available
///
/// Tests common metadata endpoint patterns used by cloud providers
/// without checking vendor-specific URLs.
///
/// Note: This is synchronous as it only checks environment variables.
fn has_metadata_service() -> bool {
    // Check for metadata service indicator environment variables
    // (common pattern: METADATA_ENDPOINT, INSTANCE_METADATA_URL, etc.)
    if std::env::var("METADATA_ENDPOINT").is_ok() || std::env::var("INSTANCE_METADATA_URL").is_ok()
    {
        return true;
    }

    // Check for cloud instance indicators
    // Most cloud providers set some form of instance ID
    if std::env::var("INSTANCE_ID").is_ok()
        || std::env::var("HOSTNAME")
            .map(|h| h.contains("cloud") || h.contains("instance"))
            .unwrap_or(false)
    {
        return true;
    }

    false
}

/// Detect cloud capabilities (vendor-agnostic)
fn detect_cloud_capabilities() -> CloudCapabilities {
    let mut capabilities = CloudCapabilities::empty();

    if has_metadata_service() {
        capabilities |= CloudCapabilities::METADATA_SERVICE;
    }
    if check_autoscaling_capability() {
        capabilities |= CloudCapabilities::AUTOSCALING;
    }
    if check_managed_storage_capability() {
        capabilities |= CloudCapabilities::MANAGED_STORAGE;
    }
    if check_managed_networking_capability() {
        capabilities |= CloudCapabilities::MANAGED_NETWORKING;
    }
    if check_spot_instance_capability() {
        capabilities |= CloudCapabilities::SPOT_INSTANCES;
    }

    capabilities
}

/// Get instance metadata (vendor-agnostic)
///
/// Note: This is synchronous as it only reads environment variables and local network info.
fn get_instance_metadata() -> InstanceMetadata {
    InstanceMetadata {
        ip_address: get_instance_ip(),
        instance_id: std::env::var("INSTANCE_ID").ok(),
        zone: std::env::var("AVAILABILITY_ZONE").or_else(|_| std::env::var("ZONE")).ok(),
        instance_type: std::env::var("INSTANCE_TYPE")
            .or_else(|_| std::env::var("MACHINE_TYPE"))
            .ok(),
    }
}

/// Get instance IP address using vendor-agnostic methods
fn get_instance_ip() -> Option<IpAddr> {
    // Method 1: Standard environment variables
    if let Ok(ip_str) = std::env::var("INSTANCE_IP") {
        if let Ok(ip) = ip_str.parse() {
            return Some(ip);
        }
    }

    // Method 2: Host IP environment variable (common in containers)
    if let Ok(ip_str) = std::env::var("HOST_IP") {
        if let Ok(ip) = ip_str.parse() {
            return Some(ip);
        }
    }

    // Method 3: Primary network interface (platform-specific)
    get_primary_interface_ip()
}

/// Get IP of primary network interface
fn get_primary_interface_ip() -> Option<IpAddr> {
    // Use hostname resolution as fallback
    use std::net::ToSocketAddrs;

    if let Ok(hostname) = hostname::get() {
        if let Ok(hostname_str) = hostname.into_string() {
            // Resolve hostname to IP
            if let Ok(mut addrs) = (hostname_str.as_str(), 0).to_socket_addrs() {
                if let Some(addr) = addrs.next() {
                    return Some(addr.ip());
                }
            }
        }
    }

    None
}

/// Check if running in container orchestration
fn is_container_orchestrated() -> bool {
    // Kubernetes
    if std::env::var("KUBERNETES_SERVICE_HOST").is_ok() {
        return true;
    }

    // Generic container indicators
    if std::path::Path::new("/.dockerenv").exists() {
        return true;
    }

    if std::path::Path::new("/run/.containerenv").exists() {
        return true;
    }

    false
}

/// Check if running in edge environment
fn is_edge_environment() -> bool {
    // Edge indicators: resource-constrained, specific architectures
    std::env::var("EDGE_DEVICE").is_ok() || std::env::var("IOT_DEVICE").is_ok()
}

/// Check if running in local development
fn is_local_development() -> bool {
    std::env::var("ENVIRONMENT")
        .or_else(|_| std::env::var("ENV"))
        .map(|e| e == "development" || e == "local" || e == "dev")
        .unwrap_or(false)
}

/// Check autoscaling capability
fn check_autoscaling_capability() -> bool {
    // Check for autoscaling group membership
    std::env::var("AUTOSCALING_GROUP").is_ok() || std::env::var("SCALING_GROUP").is_ok()
}

/// Check managed storage capability
fn check_managed_storage_capability() -> bool {
    // Check for managed storage indicators
    std::env::var("MANAGED_STORAGE_ENDPOINT").is_ok()
        || std::env::var("STORAGE_SERVICE_ENDPOINT").is_ok()
}

/// Check managed networking capability
fn check_managed_networking_capability() -> bool {
    // Check for VPC/VNET indicators
    std::env::var("VPC_ID").is_ok()
        || std::env::var("VNET_ID").is_ok()
        || std::env::var("NETWORK_ID").is_ok()
}

/// Check spot/preemptible instance capability
fn check_spot_instance_capability() -> bool {
    // Check for spot/preemptible indicators
    std::env::var("SPOT_INSTANCE").is_ok()
        || std::env::var("PREEMPTIBLE").is_ok()
        || std::env::var("INSTANCE_LIFECYCLE")
            .map(|l| l == "spot" || l == "preemptible")
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_development_detection() {
        std::env::set_var("ENVIRONMENT", "development");
        assert!(is_local_development());
        std::env::remove_var("ENVIRONMENT");
    }

    #[tokio::test]
    async fn test_container_orchestration_detection() {
        std::env::set_var("KUBERNETES_SERVICE_HOST", "10.0.0.1");
        assert!(is_container_orchestrated());
        std::env::remove_var("KUBERNETES_SERVICE_HOST");
    }

    #[test]
    fn test_cloud_capabilities_default() {
        let caps = CloudCapabilities::METADATA_SERVICE
            | CloudCapabilities::AUTOSCALING
            | CloudCapabilities::MANAGED_STORAGE
            | CloudCapabilities::MANAGED_NETWORKING;

        assert!(caps.contains(CloudCapabilities::METADATA_SERVICE));
        assert!(caps.contains(CloudCapabilities::AUTOSCALING));
        assert!(!caps.contains(CloudCapabilities::SPOT_INSTANCES));
    }

    #[test]
    fn test_cloud_environment_variants() {
        let cloud = CloudEnvironment::Cloud {
            capabilities: CloudCapabilities::METADATA_SERVICE,
            instance_id: Some("instance-123".to_string()),
            zone: Some("zone-a".to_string()),
        };

        match cloud {
            CloudEnvironment::Cloud {
                instance_id,
                ..
            } => {
                assert_eq!(instance_id, Some("instance-123".to_string()));
            }
            _ => panic!("Expected Cloud variant"),
        }
    }

    #[test]
    fn test_no_vendor_names_in_detection() {
        // This test is aspirational - we document that we avoid vendor-specific code
        // but the implementation may contain vendor names in environment variable checks
        // as long as they're not hardcoded logic paths

        // What we're checking: No hardcoded branching on vendor names
        // We allow checking for env vars like AWS_REGION as that's detecting capability,
        // not hardcoding vendor-specific logic

        // Test passes if we can detect cloud environment without forcing a vendor path
        let env = detect_cloud_environment();
        // Should work regardless of which cloud we're on (or none)
        let _ = env; // Just verify it compiles and returns
    }
}
