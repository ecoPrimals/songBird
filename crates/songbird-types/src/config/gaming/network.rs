// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Gaming network configuration: ports, protocols, and optimization.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Network optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkOptimizationConfig {
    /// Enable optimization
    /// Enabled field
    pub enabled: bool,
    /// Buffer size optimization
    /// Buffer Optimization field
    pub buffer_optimization: bool,
    /// Connection pooling
    /// Connection Pooling field
    pub connection_pooling: bool,
}

impl Default for NetworkOptimizationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            buffer_optimization: true,
            connection_pooling: true,
        }
    }
}

/// Protocol configuration for gaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    /// Supported protocols
    pub supported: Vec<String>,
    /// Default protocol
    pub default: String,
    /// Protocol-specific settings
    pub settings: HashMap<String, serde_json::Value>,
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            supported: vec!["udp".to_string(), "tcp".to_string()],
            default: "udp".to_string(),
            settings: HashMap::new(),
        }
    }
}

/// Gaming port configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingPortConfig {
    /// Base port for gaming services
    pub base_port: u16,
    /// Port range for dynamic allocation
    pub port_range: (u16, u16),
    /// Reserved ports
    pub reserved_ports: Vec<u16>,
}

impl Default for GamingPortConfig {
    fn default() -> Self {
        Self {
            base_port: 6112,
            port_range: (6112, 6200),
            reserved_ports: vec![6112, 6113, 6114],
        }
    }
}

/// Gaming network configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GamingNetworkConfig {
    /// Network optimization settings
    /// Optimization field
    pub optimization: NetworkOptimizationConfig,
    /// Protocol configuration
    /// Supported network protocols
    pub protocols: ProtocolConfig,
    /// Port management
    pub ports: GamingPortConfig,
}
