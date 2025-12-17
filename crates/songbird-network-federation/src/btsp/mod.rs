//! BearDog Secure Tunnel Protocol (BTSP) Interface
//! 
//! This module provides the interface for integrating with BearDog's genetic
//! cryptography and secure tunnel protocol. It includes:
//! 
//! - Trait definitions for BTSP providers
//! - Local implementation for testing without BearDog
//! - Integration points for real BearDog connection
//! 
//! ## Architecture
//! 
//! The BTSP system is designed for sovereignty:
//! - Songbird has self-knowledge only
//! - Discovers BearDog via capability-based discovery at runtime
//! - Gracefully degrades if BearDog unavailable
//! - No hardcoded BearDog dependencies
//! 
//! ## Testing
//! 
//! Local implementation allows testing federation with encryption without
//! requiring BearDog to be running. When BearDog is available, the real
//! provider is discovered and used automatically.

pub mod provider;
pub mod tunnel;
pub mod local;

pub use provider::{BtspProvider, BtspConfig};
pub use tunnel::{Tunnel, TunnelHandle, TunnelStatus, SecurityContext};
pub use local::LocalBtspProvider;

