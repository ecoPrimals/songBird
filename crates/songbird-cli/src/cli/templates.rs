// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Configuration templates for various deployment scenarios

/// Configuration template generator
pub struct ConfigTemplate;

impl ConfigTemplate {
    /// Generate a basic service template
    #[must_use]
    pub fn service_template() -> String {
        let cfg = songbird_types::config::CanonicalSongbirdConfig::default();
        let mut config = String::new();
        config.push_str("[orchestrator]\n");
        config.push_str(&format!("bind_address = \"{}\"\n", cfg.network.bind_host));
        config.push_str(&format!("port = {}\n", cfg.network.base_port));
        config.push_str("log_level = \"info\"\n\n");
        config.push_str("[network]\n");
        config.push_str("enable_tls = false\n");
        config.push_str("enable_http2 = true\n\n");
        config.push_str("[security]\n");
        config.push_str("enable_auth = false\n");
        config.push_str("enable_audit = false\n\n");
        config.push_str("[observability]\n");
        config.push_str("enable_dashboard = true\n");
        config.push_str("metrics_interval_secs = 30\n");
        config
    }

    /// Generate a development configuration
    #[must_use]
    pub fn development_config_template() -> String {
        let cfg = songbird_types::config::CanonicalSongbirdConfig::default();
        let mut config = String::new();
        config.push_str("[orchestrator]\n");
        config.push_str(&format!("bind_address = \"{}\"\n", cfg.network.bind_host));
        config.push_str(&format!("port = {}\n", cfg.network.base_port));
        config.push_str("log_level = \"debug\"\n");
        config.push_str("enable_metrics = true\n\n");
        config.push_str("[network]\n");
        config.push_str("enable_tls = false\n");
        config.push_str("enable_http2 = true\n\n");
        config.push_str("[security]\n");
        config.push_str("enable_auth = false\n");
        config.push_str("enable_audit = false\n\n");
        config.push_str("[observability]\n");
        config.push_str("enable_dashboard = true\n");
        config.push_str("metrics_interval_secs = 10\n");
        config
    }

    /// Generate a production configuration
    #[must_use]
    pub fn production_config_template() -> String {
        let cfg = songbird_types::config::CanonicalSongbirdConfig::default();
        let mut config = String::new();
        config.push_str("[orchestrator]\n");
        config.push_str(&format!("bind_address = \"{}\"\n", cfg.network.bind_host));
        config.push_str(&format!("port = {}\n", cfg.network.base_port));
        config.push_str("log_level = \"warn\"\n");
        config.push_str("enable_metrics = true\n\n");
        config.push_str("[network]\n");
        config.push_str("enable_tls = true\n");
        config.push_str("enable_http2 = true\n\n");
        config.push_str("[security]\n");
        config.push_str("enable_auth = true\n");
        config.push_str("enable_audit = true\n\n");
        config.push_str("[observability]\n");
        config.push_str("enable_dashboard = false\n");
        config.push_str("metrics_interval_secs = 60\n");
        config
    }

    /// Generate a home network configuration
    #[must_use]
    pub fn home_network_config_template() -> String {
        let cfg = songbird_types::config::CanonicalSongbirdConfig::default();
        let mut config = String::new();
        config.push_str("[orchestrator]\n");
        config.push_str(&format!("bind_address = \"{}\"\n", cfg.network.bind_host));
        config.push_str(&format!("port = {}\n", cfg.network.base_port));
        config.push_str("log_level = \"info\"\n\n");
        config.push_str("[network]\n");
        config.push_str("enable_discovery = true\n");
        config.push_str("enable_tls = false\n\n");
        config.push_str("[security]\n");
        config.push_str("enable_basic_security = true\n\n");
        config.push_str("[discovery]\n");
        config.push_str("enable_multicast = true\n");
        config.push_str("discovery_interval_secs = 30\n");
        config
    }

    /// Generate a simple Dockerfile template
    #[must_use]
    pub fn dockerfile_template() -> String {
        let cfg = songbird_types::config::CanonicalSongbirdConfig::default();
        let mut dockerfile = String::new();
        dockerfile.push_str("FROM rust:1.75-slim as builder\n\n");
        dockerfile.push_str("RUN apt-get update && apt-get install -y pkg-config libssl-dev\n\n");
        dockerfile.push_str("WORKDIR /app\n\n");
        dockerfile.push_str("COPY . .\n");
        dockerfile.push_str("RUN cargo build --release\n\n");
        dockerfile.push_str("FROM debian:bookworm-slim\n\n");
        dockerfile.push_str("RUN apt-get update && apt-get install -y ca-certificates curl\n\n");
        dockerfile.push_str(
            "COPY --from=builder /app/target/release/songbird /usr/local/bin/songbird\n\n",
        );
        dockerfile.push_str(&format!("EXPOSE {}\n\n", cfg.network.base_port));
        dockerfile.push_str("CMD [\"songbird\", \"start\"]\n");
        dockerfile
    }
}
