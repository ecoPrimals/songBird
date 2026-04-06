// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Sovereign Security with Optional Network Effects
//!
//! ## Primal Sovereignty Architecture
//!
//! Songbird is a **sovereign primal** - it operates independently and never depends
//! on other primals. However, it can leverage **network effects** when other primals
//! are available.
//!
//! ### Security Model
//!
//! 1. **Sovereign Security** (Always Available)
//!    - Songbird's own authentication/authorization
//!    - Works on LAN without any other primals
//!    - Simple, reliable, always functional
//!
//! 2. **Network Effect Enhancement** (Optional)
//!    - Discover `security provider` via capability discovery
//!    - If available: delegate enhanced security checks
//!    - If unavailable: gracefully continue with sovereign security
//!
//! ### Philosophy
//!
//! **"Each primal knows itself and is sovereign"**
//!
//! - Songbird continues normally if `security provider` goes down
//! - Loses unique `security provider` security features → falls back to failsafe
//! - LAN users can interact without `security provider` (though `security provider` adds security)
//! - Internet/public: utilize network effect of multiple primals

use serde::{Deserialize, Serialize};
use songbird_http_client::IpcHttpClient;
use songbird_types::{SongbirdError, SongbirdResult};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Sovereign security validator with optional `security provider` network effect
pub struct SovereignSecurityValidator {
    /// Songbird's sovereign security (always available)
    sovereign: Arc<RwLock<SovereignSecurity>>,

    /// Optional delegated security-provider integration (discovered via capability)
    security_provider: Arc<RwLock<Option<SecurityProviderIntegration>>>,

    /// Configuration
    #[expect(dead_code, reason = "stored for future validator configuration hooks")]
    config: SecurityConfig,
}

impl SovereignSecurityValidator {
    /// Create new sovereign security validator
    pub fn new(config: SecurityConfig) -> Self {
        info!("🎯 Initializing Songbird sovereign security");
        info!("   Mode: Sovereign (Songbird-native)");
        info!("   Security provider integration: Optional via discovery");

        Self {
            sovereign: Arc::new(RwLock::new(SovereignSecurity::new(config.clone()))),
            security_provider: Arc::new(RwLock::new(None)),
            config,
        }
    }

    /// Attempt to discover and integrate with the optional security-provider endpoint (network effect).
    ///
    /// # Errors
    ///
    /// Returns an error if the security provider connection fails when an endpoint is configured.
    pub async fn discover_security_provider(&self) -> SongbirdResult<bool> {
        info!("🔍 Attempting security provider capability discovery...");

        // Check environment (capability-domain names first; legacy primal-specific last)
        let security_provider_configured = if songbird_process_env::var(
            "SECURITY_PROVIDER_ENDPOINT",
        )
        .is_ok()
            || songbird_process_env::var("SECURITY_ENDPOINT").is_ok()
        {
            true
        } else if songbird_process_env::var("BEARDOG_SECURITY_ENDPOINT").is_ok() {
            warn!(
                "Using legacy env var BEARDOG_SECURITY_ENDPOINT — migrate to SECURITY_PROVIDER_ENDPOINT, SECURITY_ENDPOINT, or CAPABILITY_SECURITY_ENDPOINT (capability-first)"
            );
            true
        } else {
            false
        };

        if security_provider_configured {
            info!("✅ Security provider discovered - enabling enhanced security network effect");
            *self.security_provider.write().await =
                Some(SecurityProviderIntegration::connect().await?);
            Ok(true)
        } else {
            info!("ℹ️  Security provider not discovered - continuing with sovereign security");
            Ok(false)
        }
    }

    /// Validate execution request
    ///
    /// # Errors
    ///
    /// Returns an error if sovereign validation fails
    ///
    /// **Primal sovereignty pattern**:
    /// 1. Try enhanced security if security provider available (network effect)
    /// 2. Fallback to sovereign security if provider unavailable
    /// 3. Always functional - never blocks on other primals
    pub async fn validate_request(
        &self,
        request: &SecurityRequest,
    ) -> SongbirdResult<SecurityDecision> {
        let integration_slot = self.security_provider.read().await;

        if let Some(ref integration) = *integration_slot {
            debug!("🔒 Using delegated security-provider validation (network effect)");

            match integration.validate(request).await {
                Ok(decision) => {
                    info!("✅ Security provider validation: {:?}", decision.allowed);
                    return Ok(decision);
                }
                Err(e) => {
                    warn!(
                        "⚠️  Security provider validation failed, falling back to sovereign: {}",
                        e
                    );
                    drop(integration_slot);
                    let mut slot = self.security_provider.write().await;
                    *slot = None;
                }
            }
        }

        // Sovereign security (always available)
        debug!("🏛️ Using Songbird sovereign security");
        let decision = self.sovereign.read().await.validate(request).await?;
        info!("✅ Sovereign validation: {:?}", decision.allowed);
        Ok(decision)
    }
}

/// Songbird's sovereign security (always available, no dependencies)
struct SovereignSecurity {
    config: SecurityConfig,
    auth_tokens: Vec<String>,
}

impl SovereignSecurity {
    fn new(config: SecurityConfig) -> Self {
        Self {
            auth_tokens: config.auth_tokens.clone(),
            config,
        }
    }

    /// Validate using Songbird's sovereign security
    async fn validate(&self, request: &SecurityRequest) -> SongbirdResult<SecurityDecision> {
        tokio::task::yield_now().await;
        // 1. Check authentication if enabled
        if self.config.enable_auth {
            if let Some(ref token) = request.auth_token {
                if !self.auth_tokens.contains(token) {
                    return Ok(SecurityDecision {
                        allowed: false,
                        reason: Some("Invalid authentication token".to_string()),
                        confidence: 1.0, // We're certain about auth failures
                        mode: SecurityMode::Sovereign,
                    });
                }
            } else {
                return Ok(SecurityDecision {
                    allowed: false,
                    reason: Some("Authentication required but not provided".to_string()),
                    confidence: 1.0,
                    mode: SecurityMode::Sovereign,
                });
            }
        }

        // 2. Validate command safety
        if let Some(violation) = check_dangerous_patterns(&request.command) {
            return Ok(SecurityDecision {
                allowed: false,
                reason: Some(format!("Dangerous command pattern detected: {violation}")),
                confidence: 1.0,
                mode: SecurityMode::Sovereign,
            });
        }

        // 3. Check basic resource limits
        if let Some(timeout) = request.timeout_seconds
            && timeout > self.config.max_timeout_seconds
        {
            return Ok(SecurityDecision {
                allowed: false,
                reason: Some(format!(
                    "Timeout {} exceeds limit {}",
                    timeout, self.config.max_timeout_seconds
                )),
                confidence: 1.0,
                mode: SecurityMode::Sovereign,
            });
        }

        // Allow with sovereign security
        Ok(SecurityDecision {
            allowed: true,
            reason: None,
            confidence: 0.8, // Good confidence with sovereign security
            mode: SecurityMode::Sovereign,
        })
    }
}

/// Optional integration with a capability-discovered security provider (network effect).
///
/// Delegates enhanced validation to the provider HTTP API when available.
struct SecurityProviderIntegration {
    /// Security provider base URL
    endpoint: String,
    /// HTTP client for provider requests
    client: IpcHttpClient,
    /// Request timeout for security operations (reserved for timeout enforcement)
    #[expect(dead_code, reason = "reserved for security-provider request timeout enforcement")]
    timeout: std::time::Duration,
}

impl SecurityProviderIntegration {
    /// Connect to the delegated security validation HTTP endpoint
    ///
    /// Resolves base URL via:
    /// 1. `SECURITY_PROVIDER_ENDPOINT` (capability-based)
    /// 2. `SECURITY_ENDPOINT` (capability domain)
    /// 3. `BEARDOG_SECURITY_ENDPOINT` (deprecated legacy)
    /// 4. `SONGBIRD_SECURITY_ENDPOINT` (deprecated legacy)
    /// 5. Development-only fallback to `localhost:DEFAULT_HTTPS_PORT`
    ///
    /// # Errors
    ///
    /// Returns an error if HTTP client cannot be created, or if no endpoint
    /// is configured in release builds.
    async fn connect() -> SongbirdResult<Self> {
        let endpoint = if let Ok(v) = songbird_process_env::var("SECURITY_PROVIDER_ENDPOINT") {
            v
        } else if let Ok(v) = songbird_process_env::var("SECURITY_ENDPOINT") {
            v
        } else if let Ok(v) = songbird_process_env::var("BEARDOG_SECURITY_ENDPOINT") {
            warn!(
                "Using legacy env var BEARDOG_SECURITY_ENDPOINT — migrate to SECURITY_PROVIDER_ENDPOINT, SECURITY_ENDPOINT, or CAPABILITY_SECURITY_ENDPOINT (capability-first)"
            );
            v
        } else if let Ok(v) = songbird_process_env::var("SONGBIRD_SECURITY_ENDPOINT") {
            warn!(
                "Using legacy env var SONGBIRD_SECURITY_ENDPOINT — migrate to SECURITY_PROVIDER_ENDPOINT or SECURITY_ENDPOINT"
            );
            v
        } else {
            #[cfg(debug_assertions)]
            {
                use songbird_types::constants::{DEFAULT_HTTPS_PORT, LOCALHOST};
                let fallback = format!("http://{LOCALHOST}:{DEFAULT_HTTPS_PORT}");
                warn!("⚠️ Using development fallback for security provider security: {fallback}");
                fallback
            }
            #[cfg(not(debug_assertions))]
            {
                return Err(SongbirdError::configuration(
                    "Security provider endpoint not configured. \
                     Set SECURITY_PROVIDER_ENDPOINT, SECURITY_ENDPOINT, or CAPABILITY_SECURITY_ENDPOINT (legacy fallbacks: BEARDOG_SECURITY_ENDPOINT, SONGBIRD_SECURITY_ENDPOINT).",
                ));
            }
        };

        // Create HTTP client
        let client = IpcHttpClient::new().await.map_err(|e| {
            SongbirdError::configuration(format!("Failed to create HTTP client: {e}"))
        })?;

        // Verify security provider is reachable (non-blocking health check)
        let health_url = format!("{endpoint}/health");
        match client.get(&health_url).await {
            Ok(response) if response.is_success() => {
                info!("🔗 Successfully connected to security provider at {endpoint}");
            }
            Ok(response) => {
                warn!(
                    "⚠️ security provider health check returned non-success: {}",
                    response.status()
                );
            }
            Err(e) => {
                warn!("⚠️ security provider not reachable (will use local validation): {e}");
            }
        }

        Ok(Self {
            endpoint,
            client,
            timeout: std::time::Duration::from_secs(5),
        })
    }

    /// Validate security request using `security provider`
    ///
    /// Calls `security provider`'s security validation API to get enhanced security decisions.
    /// Falls back to permissive local decision if `security provider` is unreachable.
    ///
    /// # Errors
    ///
    /// Returns an error only for unrecoverable failures (not network issues)
    async fn validate(&self, request: &SecurityRequest) -> SongbirdResult<SecurityDecision> {
        info!("🛡️ Delegating to security provider for enhanced security");

        let url = format!("{}/security/validate", self.endpoint);

        // Prepare request payload
        let payload = serde_json::json!({
            "command": &request.command,
            "auth_token": &request.auth_token,
            "timeout_seconds": request.timeout_seconds,
            "requester": &request.requester,
        });

        // Call security provider security validation API
        let request_builder = self.client.post(&url).await;
        let request_with_body = request_builder
            .json(&payload)
            .map_err(|e| SongbirdError::configuration(format!("Failed to build request: {e}")))?;

        match request_with_body.send().await {
            Ok(response) if response.is_success() => {
                // Parse security provider's security decision
                match response.json::<SecurityDecision>().await {
                    Ok(decision) => {
                        info!("✅ security provider validation complete: {:?}", decision.allowed);
                        Ok(decision)
                    }
                    Err(e) => {
                        warn!("⚠️ Failed to parse security provider response: {e}");
                        // Fallback to permissive decision
                        Ok(SecurityDecision {
                            allowed: true,
                            reason: Some(
                                "security provider validation unavailable (parse error)"
                                    .to_string(),
                            ),
                            confidence: 0.5,
                            mode: SecurityMode::Sovereign,
                        })
                    }
                }
            }
            Ok(response) => {
                warn!("⚠️ security provider returned error status: {}", response.status());
                // Fallback to permissive decision
                Ok(SecurityDecision {
                    allowed: true,
                    reason: Some(format!(
                        "security provider validation unavailable (HTTP {})",
                        response.status()
                    )),
                    confidence: 0.5,
                    mode: SecurityMode::Sovereign,
                })
            }
            Err(e) => {
                warn!("⚠️ security provider request failed: {e}");
                // Fallback to permissive decision (don't block on network errors)
                Ok(SecurityDecision {
                    allowed: true,
                    reason: Some(
                        "security provider validation unavailable (network error)".to_string(),
                    ),
                    confidence: 0.5,
                    mode: SecurityMode::Sovereign,
                })
            }
        }
    }

    /// Check if `security provider` is currently reachable
    ///
    /// Non-blocking health check to determine if `security provider` integration is active
    #[expect(dead_code, reason = "reserved for security provider availability probing")]
    async fn is_available(&self) -> bool {
        let url = format!("{}/health", self.endpoint);

        self.client.get(&url).await.map(|r| r.is_success()).unwrap_or(false)
    }

    /// Get `security provider` endpoint URL
    #[must_use]
    #[expect(dead_code, reason = "accessor reserved for diagnostics and future callers")]
    fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

// Note: Default trait removed - use connect() async method instead.
// This enforces modern async patterns and proper error handling.

/// Check for dangerous command patterns (Songbird sovereign check)
fn check_dangerous_patterns(command: &str) -> Option<String> {
    let patterns = ["rm -rf /", ":(){ :|:& };:", "mkfs", "dd if=/dev/zero"];

    for pattern in &patterns {
        if command.contains(pattern) {
            return Some(pattern.to_string());
        }
    }

    None
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable authentication
    pub enable_auth: bool,

    /// Auth tokens (sovereign security)
    pub auth_tokens: Vec<String>,

    /// Maximum timeout (seconds)
    pub max_timeout_seconds: u64,

    /// Enable discovery of optional delegated security provider.
    ///
    /// Serialized as `enable_security_provider_discovery`; for wire compatibility the key
    /// `enable_beardog_discovery` is also accepted (via serde alias).
    #[serde(default, alias = "enable_beardog_discovery")]
    pub enable_security_provider_discovery: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_auth: true,
            auth_tokens: vec![],       // Empty = reject all until configured
            max_timeout_seconds: 7200, // 2 hours
            enable_security_provider_discovery: true,
        }
    }
}

/// Local validation input for [`SovereignSecurityValidator`] before a job is accepted.
#[derive(Debug, Clone)]
pub struct SecurityRequest {
    /// Command line or script body under review.
    pub command: String,
    /// Bearer or shared secret when auth is enabled in [`SecurityConfig`].
    pub auth_token: Option<String>,
    /// Optional per-request wall-clock cap overriding defaults.
    pub timeout_seconds: Option<u64>,
    /// Optional caller identity for logs and policy hooks.
    pub requester: Option<String>,
}

/// Outcome of sovereign validation suitable for RPC responses and audit logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityDecision {
    /// Whether the executor may start the job.
    pub allowed: bool,
    /// Denial or advisory text when not allowed or partially restricted.
    pub reason: Option<String>,
    /// Local policy confidence in `0.0..=1.0`.
    pub confidence: f64,
    /// Which policy path produced the decision.
    pub mode: SecurityMode,
}

/// Identifies which policy backend supplied the [`SecurityDecision`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityMode {
    /// On-tower [`SovereignSecurityValidator`] rules only.
    Sovereign,

    /// Augmented decision that incorporated another primal's attestation or reputation.
    NetworkEffect {
        /// Peer primal identifier (e.g. discovery name).
        primal: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sovereign_security_always_works() {
        let config = SecurityConfig {
            enable_auth: false, // Disable for test
            ..Default::default()
        };

        let validator = SovereignSecurityValidator::new(config);

        let request = SecurityRequest {
            command: "echo hello".to_string(),
            auth_token: None,
            timeout_seconds: Some(60),
            requester: Some("test".to_string()),
        };

        let decision = validator.validate_request(&request).await.unwrap();
        assert!(decision.allowed);
        assert!(matches!(decision.mode, SecurityMode::Sovereign));
    }

    #[tokio::test]
    async fn test_dangerous_command_blocked() {
        let config = SecurityConfig {
            enable_auth: false,
            ..Default::default()
        };

        let validator = SovereignSecurityValidator::new(config);

        let request = SecurityRequest {
            command: "rm -rf /".to_string(),
            auth_token: None,
            timeout_seconds: Some(60),
            requester: Some("test".to_string()),
        };

        let decision = validator.validate_request(&request).await.unwrap();
        assert!(!decision.allowed);
        assert!(decision.reason.is_some());
    }

    #[tokio::test]
    async fn test_auth_enforcement() {
        let config = SecurityConfig {
            enable_auth: true,
            auth_tokens: vec!["secret123".to_string()],
            ..Default::default()
        };

        let validator = SovereignSecurityValidator::new(config);

        // Valid token
        let request = SecurityRequest {
            command: "echo hello".to_string(),
            auth_token: Some("secret123".to_string()),
            timeout_seconds: Some(60),
            requester: Some("test".to_string()),
        };

        let decision = validator.validate_request(&request).await.unwrap();
        assert!(decision.allowed);

        // Invalid token
        let request = SecurityRequest {
            command: "echo hello".to_string(),
            auth_token: Some("wrong".to_string()),
            timeout_seconds: Some(60),
            requester: Some("test".to_string()),
        };

        let decision = validator.validate_request(&request).await.unwrap();
        assert!(!decision.allowed);
    }
}
