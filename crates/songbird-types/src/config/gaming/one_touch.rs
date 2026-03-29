// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! One-touch gaming setup: profiles and templates.

use super::taxonomy::{GameProtocolClass, PerformanceMode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Gaming profile for one-touch setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingProfile {
    /// Profile name
    /// Name identifier
    pub name: String,
    /// Gaming protocol preference
    /// Protocol Preference field
    pub protocol_preference: Vec<GameProtocolClass>,
    /// Performance settings
    /// Performance Mode field
    pub performance_mode: PerformanceMode,
}

impl Default for GamingProfile {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            protocol_preference: vec![GameProtocolClass::RealTimeStrategy],
            performance_mode: PerformanceMode::Balanced,
        }
    }
}

/// Gaming template for quick setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingTemplate {
    /// Template name
    /// Name identifier
    pub name: String,
    /// Port configuration
    pub ports: Vec<u16>,
    /// Protocol settings
    /// Supported network protocols
    pub protocols: Vec<GameProtocolClass>,
}

/// One-touch configuration - consolidates `OneTouchConfig`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneTouchConfig {
    /// Enable one-touch setup
    /// Enabled field
    pub enabled: bool,
    /// Default gaming profile
    pub default_profile: GamingProfile,
    /// Quick setup templates
    pub templates: HashMap<String, GamingTemplate>,
}

impl Default for OneTouchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_profile: GamingProfile::default(),
            templates: HashMap::new(),
        }
    }
}
