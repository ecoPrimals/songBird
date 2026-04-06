// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Auto-configuration, security provider integration, and protocol detection.

use super::network::NetworkOptimizationConfig;
use super::performance::GamingPerformanceSettings;
use super::security::{GamingAuthConfig, GamingSecuritySettings};
use super::taxonomy::GameProtocolClass;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Security provider monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityProviderMonitoringConfig {
    /// Enable monitoring
    /// Enabled field
    pub enabled: bool,
    /// Metrics collection interval in seconds
    /// Metrics Interval field
    pub metrics_interval: u32,
    /// Health check interval in seconds
    /// Health Check Interval field
    pub health_check_interval: u32,
}

impl Default for SecurityProviderMonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            metrics_interval: 60,
            health_check_interval: 30,
        }
    }
}

/// Security provider integration configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityProviderIntegrationConfig {
    /// Enable `security_provider_config` integration
    /// Enabled field
    pub enabled: bool,
    /// `security_provider` endpoint
    /// Endpoint field
    pub endpoint: Option<String>,
    /// Authentication settings
    pub auth: GamingAuthConfig,
    /// Security settings
    pub security: GamingSecuritySettings,
    /// Performance settings
    /// Performance field
    pub performance: GamingPerformanceSettings,
    /// Monitoring settings
    /// Monitoring field
    pub monitoring: SecurityProviderMonitoringConfig,
}

/// Auto-detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoDetectionConfig {
    /// Enable auto-detection
    /// Enabled field
    pub enabled: bool,
    /// Detection timeout in seconds
    /// Timeout Seconds field
    pub timeout_seconds: u32,
    /// Detection interval in seconds
    /// Interval Seconds field
    pub interval_seconds: u32,
}

impl Default for AutoDetectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_seconds: 10,
            interval_seconds: 30,
        }
    }
}

/// Gaming auto-configuration - consolidates `GamingAutoConfig`s
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingAutoConfig {
    /// Enable auto-configuration
    /// Enabled field
    pub enabled: bool,
    /// Security provider integration settings
    /// Security Provider Config field
    pub security_provider_config: SecurityProviderIntegrationConfig,
    /// Auto-detection settings
    /// Detection field
    pub detection: AutoDetectionConfig,
    /// Network optimization settings
    /// Optimization field
    pub optimization: NetworkOptimizationConfig,
}

impl Default for GamingAutoConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            security_provider_config: SecurityProviderIntegrationConfig::default(),
            detection: AutoDetectionConfig::default(),
            optimization: NetworkOptimizationConfig::default(),
        }
    }
}

/// Protocol detection configuration - consolidates detection configs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolDetectionConfig {
    /// Enable protocol detection
    /// Enabled field
    pub enabled: bool,
    /// Detection timeout
    /// Detection Timeout field
    pub detection_timeout: Duration,
    /// Supported protocols
    pub supported_protocols: Vec<GameProtocolClass>,
    /// Detection rules
    pub detection_rules: Vec<DetectionRule>,
}

impl Default for ProtocolDetectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            detection_timeout: Duration::from_secs(5),
            supported_protocols: vec![
                GameProtocolClass::RealTimeStrategy,
                GameProtocolClass::FirstPersonShooter,
            ],
            detection_rules: Vec::new(),
        }
    }
}

/// Protocol detection rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionRule {
    /// Rule name
    /// Name identifier
    pub name: String,
    /// Protocol signature bytes when matched
    /// Signature field
    pub signature: Option<Vec<u8>>,
    /// Target protocol class
    pub protocol_class: GameProtocolClass,
}
