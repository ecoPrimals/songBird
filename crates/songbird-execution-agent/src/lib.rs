// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::clone_on_ref_ptr,
    reason = "Arc::clone() is idiomatic for shared ownership in async contexts"
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions"))]
//! # Songbird Execution Agent
//!
//! Remote command execution agent that runs on each tower in the federation.
//! Receives execution requests from the orchestrator and manages process lifecycle.
//!
//! ## Features
//! - Remote command execution
//! - Background process management
//! - Job tracking and monitoring
//! - Output capture (stdout/stderr)
//! - Security and resource limits
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod executor;
pub mod job_manager;
pub mod security;
pub mod security_provider;

/// Deprecated alias for [`security_provider`].
#[deprecated(note = "use module security_provider")]
pub mod security_provider_legacy {
    pub use crate::security_provider::*;
}
pub mod security_sovereign;
pub mod server;
pub mod types;

pub use executor::CommandExecutor;
pub use job_manager::JobManager;
pub use security_sovereign::{
    SecurityConfig, SecurityDecision, SecurityRequest, SovereignSecurityValidator,
};
pub use server::ExecutionServer;
pub use types::*;

use songbird_types::SongbirdResult;

/// Agent configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentConfig {
    /// Port to listen on
    pub port: u16,

    /// Bind address
    pub bind_address: String,

    /// Maximum concurrent jobs
    pub max_concurrent_jobs: usize,

    /// Job log retention (seconds)
    pub log_retention_seconds: u64,

    /// Enable authentication
    pub enable_auth: bool,

    /// Auth token (if enabled)
    pub auth_token: Option<String>,

    /// Resource limits
    pub resource_limits: ResourceLimits,
}

/// Caps memory, CPU time, and default wall-clock timeout applied by [`CommandExecutor`] to each job.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory per job (MB)
    pub max_memory_mb: Option<u64>,

    /// Maximum CPU time per job (seconds)
    pub max_cpu_time_seconds: Option<u64>,

    /// Default timeout (seconds)
    pub default_timeout_seconds: u64,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            port: 9020,
            bind_address: "0.0.0.0".to_string(),
            max_concurrent_jobs: 100,
            log_retention_seconds: 86400, // 24 hours
            enable_auth: true,
            auth_token: None,
            resource_limits: ResourceLimits::default(),
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: Some(4096),        // 4GB
            max_cpu_time_seconds: Some(3600), // 1 hour
            default_timeout_seconds: 3600,
        }
    }
}

/// Initialize agent with configuration
///
/// # Errors
///
/// Currently infallible but returns Result for future extensibility
pub async fn init_agent(config: AgentConfig) -> SongbirdResult<ExecutionServer> {
    tokio::task::yield_now().await;
    tracing::info!("Initializing execution agent on {}:{}", config.bind_address, config.port);

    let job_manager = JobManager::new(config.max_concurrent_jobs, config.log_retention_seconds);
    let executor = CommandExecutor::new(config.resource_limits.clone());

    let server = ExecutionServer::new(
        config.bind_address.clone(),
        config.port,
        job_manager,
        executor,
        config.enable_auth.then(|| config.auth_token.clone()).flatten(),
    );

    Ok(server)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AgentConfig::default();
        assert_eq!(config.port, 9020);
        assert_eq!(config.max_concurrent_jobs, 100);
        assert!(config.enable_auth);
    }
}
