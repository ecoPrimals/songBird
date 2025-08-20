//! Security Tunnel Implementations
//!
//! Tunnel implementations for different security providers

use super::types::{SecureTunnel, TunnelStatus, TunnelType};
use async_trait::async_trait;
use songbird_errors::Result;
use std::time::Instant;
use tracing::info;

/// No-op tunnel for testing
pub struct NoOpTunnel;

#[async_trait]
impl SecureTunnel for NoOpTunnel {
    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // No-op encryption for testing
        Ok(data.to_vec())
    }

    async fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // No-op decryption for testing
        Ok(data.to_vec())
    }

    async fn get_status(&self) -> Result<TunnelStatus> {
        Ok(TunnelStatus {
            is_active: true,
            bytes_sent: 0,
            bytes_received: 0,
            last_activity: Some(Instant::now()),
            latency: None,
        })
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
} 