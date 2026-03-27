// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::clone_on_ref_ptr,
    reason = "Arc::clone() is idiomatic for shared ownership in async contexts"
)]
#![cfg_attr(
    test,
    allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")
)]
//! # Songbird IGD - Sovereign Router Configuration
//!
//! Pure Rust implementation of `UPnP` IGD (Internet Gateway Device) and NAT-PMP protocols
//! for automatic router port forwarding configuration.
//!
//! ## Overview
//!
//! Turns the router from a manual dependency into a tool Songbird configures automatically.
//! This crate enables Songbird to:
//!
//! - Discover routers via SSDP multicast (`UPnP`) or NAT-PMP probes
//! - Request port mappings programmatically
//! - Query external IP addresses from the router
//! - Manage mapping TTLs and automatic renewal
//! - Provide clear fallback guidance when auto-config isn't available
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────┐
//! │   Gateway   │  ← Unified abstraction
//! └──────┬──────┘
//!        │
//!   ┌────┴────┐
//!   │         │
//! ┌─▼──┐  ┌──▼──┐
//! │SSDP│  │NAT- │
//! │SOAP│  │ PMP │
//! └────┘  └─────┘
//! ```
//!
//! ## Protocols Supported
//!
//! - **`UPnP` IGD** (RFC 6970): SSDP discovery + SOAP control
//! - **NAT-PMP** (RFC 6886): Simple binary UDP protocol
//! - **Manual fallback**: Clear instructions when auto-config unavailable
//!
//! ## Deep Debt Compliance
//!
//! - ✅ 100% Pure Rust (zero C dependencies)
//! - ✅ Zero unsafe code
//! - ✅ Modern idiomatic async/await
//! - ✅ Protocol implementations from scratch (no external protocol crates)
//! - ✅ Runtime discovery (zero hardcoded values)
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use songbird_igd::Gateway;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Discover router
//!     let gateway = Gateway::discover().await?;
//!     
//!     // Map port 3492 for Songbird
//!     let mapping = gateway.map_port(3492, 3492, "TCP", 86400).await?;
//!     
//!     println!("Port forwarded: {:?}", mapping);
//!     
//!     // Get external IP
//!     let external_ip = gateway.get_external_ip().await?;
//!     println!("External IP: {}", external_ip);
//!     
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod gateway;
pub mod mapping;
pub mod nat_pmp;
pub mod renewal;
pub mod soap;
pub mod ssdp;

pub use error::{IgdError, Result};
pub use gateway::{Gateway, GatewayProtocol};
pub use mapping::{PortMapping, PortMappingRequest, Protocol};
pub use nat_pmp::NatPmpClient;
pub use soap::SoapClient;
pub use ssdp::{SsdpClient, SsdpResponse};

/// Version of the IGD implementation
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default SSDP multicast address
pub const SSDP_MULTICAST_ADDR: &str = "239.255.255.250:1900";

/// Default NAT-PMP port
pub const NAT_PMP_PORT: u16 = 5351;

/// Default port mapping TTL (24 hours)
pub const DEFAULT_MAPPING_TTL: u32 = 86400;

/// `UPnP` IGD device type
pub const IGD_DEVICE_TYPE: &str = "urn:schemas-upnp-org:device:InternetGatewayDevice:1";

/// `UPnP` `WANIPConnection` service type
pub const WANIP_SERVICE_TYPE: &str = "urn:schemas-upnp-org:service:WANIPConnection:1";
