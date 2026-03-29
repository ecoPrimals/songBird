// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Gaming security, encryption, authentication, and privilege settings.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Gaming security settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingSecuritySettings {
    /// Enable security features
    /// Enabled field
    pub enabled: bool,
    /// Anti-cheat enabled
    /// Anti Cheat field
    pub anti_cheat: bool,
    /// Encryption enabled
    /// Whether encryption is enabled
    pub encryption: bool,
}

impl Default for GamingSecuritySettings {
    fn default() -> Self {
        Self {
            enabled: true,
            anti_cheat: true,
            encryption: true,
        }
    }
}

/// Gaming authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingAuthConfig {
    /// Enable authentication
    /// Enabled field
    pub enabled: bool,
    /// Authentication method
    pub method: String,
    /// Session timeout in seconds
    /// Session Timeout field
    pub session_timeout: u64,
}

impl Default for GamingAuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            method: "jwt".to_string(),
            session_timeout: 3600,
        }
    }
}

/// Gaming security configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GamingSecurityConfig {
    /// Security settings
    /// Settings field
    pub settings: GamingSecuritySettings,
    /// Authentication configuration
    pub auth: GamingAuthConfig,
}

/// Encryption configuration for gaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Enable encryption
    pub enabled: bool,
    /// Encryption algorithm
    pub algorithm: String,
    /// Key size
    pub key_size: u32,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: "AES256".to_string(),
            key_size: 256,
        }
    }
}

/// Authentication configuration for gaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    /// Enable authentication
    /// Enabled field
    pub enabled: bool,
    /// Authentication method
    pub method: String,
    /// Token lifetime
    /// Token Lifetime field
    pub token_lifetime: Duration,
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            method: "bearer_token".to_string(),
            token_lifetime: Duration::from_secs(3600),
        }
    }
}

/// Privilege configuration for gaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivilegeConfig {
    /// Enable privilege management
    pub enabled: bool,
    /// Default privilege level
    pub default_level: u32,
    /// Maximum privilege level
    pub max_level: u32,
}

impl Default for PrivilegeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_level: 1,
            max_level: 10,
        }
    }
}
