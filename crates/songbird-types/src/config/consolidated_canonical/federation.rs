// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # Federation Configuration Module
//!
//! **CANONICAL FEDERATION CONFIGURATION** ✅
//!
//! This module provides federation and clustering configuration structures for the Songbird ecosystem.
//! Uses idiomatic Rust patterns: enums for policies, bitflags for features.

use serde::{Deserialize, Serialize};

// ============================================================================
// FEDERATION CONFIGURATION - Zero-Trust Federation
// ============================================================================

/// Federation acceptance policy - replaces `auto_accept_lan/wan` bools
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum FederationAcceptancePolicy {
    /// Manual approval required for all connections
    ManualOnly,
    /// Auto-accept from LAN, manual for WAN
    #[default]
    LanAutoWanManual,
    /// Auto-accept from both LAN and WAN
    AutoAcceptAll,
}

/// Trust escalation policy - replaces multiple escalation bools
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum TrustEscalationPolicy {
    /// No escalation allowed (locked at initial trust level)
    Disabled,
    /// Only capability escalation allowed
    CapabilityOnly,
    /// Both capability and identity escalation allowed
    #[default]
    Progressive,
}

/// **CANONICAL**: Federation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalFederationConfig {
    /// Cluster name (default: auto-detected from hostname, None = disabled)
    pub cluster_name: Option<String>,

    /// Trust escalation policy (replaces multiple bools)
    pub trust_escalation_policy: TrustEscalationPolicy,

    /// Initial trust level for new federation members (default: anonymous)
    pub initial_trust_level: String,

    /// Require hardware key for admin operations (default: true)
    pub require_hardware_for_admin: bool,

    /// Federation acceptance policy (replaces `auto_accept` bools)
    pub acceptance_policy: FederationAcceptancePolicy,

    /// Trust timeouts for different trust levels (in seconds)
    pub trust_timeouts: TrustTimeouts,
}

/// Trust timeouts for progressive escalation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustTimeouts {
    /// Anonymous sessions expire after this (default: 3600 = 1 hour)
    pub anonymous: u64,
    /// Capability sessions expire after this (default: 86400 = 24 hours)
    pub capability: u64,
    /// Identity sessions expire after this (default: 604800 = 7 days)
    pub identity: u64,
    /// Hardware sessions never expire (default: 0 = never)
    pub hardware: u64,
}

impl Default for CanonicalFederationConfig {
    fn default() -> Self {
        let trust_escalation_policy = songbird_process_env::var("SONGBIRD_TRUST_ESCALATION_POLICY")
            .ok()
            .and_then(|v| {
                let v = v.trim();
                if v.eq_ignore_ascii_case("disabled") {
                    Some(TrustEscalationPolicy::Disabled)
                } else if v.eq_ignore_ascii_case("capability") {
                    Some(TrustEscalationPolicy::CapabilityOnly)
                } else if v.eq_ignore_ascii_case("progressive") {
                    Some(TrustEscalationPolicy::Progressive)
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let acceptance_policy = songbird_process_env::var("SONGBIRD_FEDERATION_ACCEPTANCE")
            .ok()
            .and_then(|v| {
                let v = v.trim();
                if v.eq_ignore_ascii_case("manual") {
                    Some(FederationAcceptancePolicy::ManualOnly)
                } else if v.eq_ignore_ascii_case("lan_auto") {
                    Some(FederationAcceptancePolicy::LanAutoWanManual)
                } else if v.eq_ignore_ascii_case("auto_all") {
                    Some(FederationAcceptancePolicy::AutoAcceptAll)
                } else {
                    None
                }
            })
            .unwrap_or_default();

        Self {
            cluster_name: songbird_process_env::var("SONGBIRD_CLUSTER_NAME")
                .ok()
                .or_else(|| gethostname::gethostname().into_string().ok()),
            trust_escalation_policy,
            initial_trust_level: String::from("anonymous"),
            require_hardware_for_admin: songbird_process_env::var(
                "SONGBIRD_REQUIRE_HARDWARE_ADMIN",
            )
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(true),
            acceptance_policy,
            trust_timeouts: TrustTimeouts::default(),
        }
    }
}

impl Default for TrustTimeouts {
    fn default() -> Self {
        Self {
            anonymous: 3_600,   // 1 hour
            capability: 86_400, // 24 hours
            identity: 604_800,  // 7 days
            hardware: 0,        // Never expire
        }
    }
}
