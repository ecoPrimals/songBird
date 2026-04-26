// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Configuration templates for various deployment scenarios

use std::fmt::Write;

/// Configuration template generator
pub struct ConfigTemplate;

impl ConfigTemplate {
    /// Generate a basic service template
    #[must_use]
    pub fn service_template() -> String {
        let cfg = songbird_types::config::CanonicalSongbirdConfig::default();
        let mut config = String::new();
        config.push_str("[orchestrator]\n");
        let _ = writeln!(config, "bind_address = \"{}\"", cfg.network.bind_host);
        let _ = writeln!(config, "port = {}", cfg.network.base_port);
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
        let _ = writeln!(config, "bind_address = \"{}\"", cfg.network.bind_host);
        let _ = writeln!(config, "port = {}", cfg.network.base_port);
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
        let _ = writeln!(config, "bind_address = \"{}\"", cfg.network.bind_host);
        let _ = writeln!(config, "port = {}", cfg.network.base_port);
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
        let _ = writeln!(config, "bind_address = \"{}\"", cfg.network.bind_host);
        let _ = writeln!(config, "port = {}", cfg.network.base_port);
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
        let _ = write!(dockerfile, "EXPOSE {}\n\n", cfg.network.base_port);
        dockerfile.push_str("CMD [\"songbird\", \"start\"]\n");
        dockerfile
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::ConfigTemplate;
    use songbird_types::config::CanonicalSongbirdConfig;

    #[test]
    fn service_template_includes_default_bind_and_port() {
        let cfg = CanonicalSongbirdConfig::default();
        let t = ConfigTemplate::service_template();
        assert!(t.contains("[orchestrator]"));
        assert!(t.contains(&format!("bind_address = \"{}\"", cfg.network.bind_host)));
        assert!(t.contains(&format!("port = {}", cfg.network.base_port)));
        assert!(t.contains("log_level = \"info\""));
        assert!(t.contains("enable_tls = false"));
    }

    #[test]
    fn development_template_uses_debug_and_shorter_metrics_interval() {
        let t = ConfigTemplate::development_config_template();
        assert!(t.contains("log_level = \"debug\""));
        assert!(t.contains("metrics_interval_secs = 10"));
    }

    #[test]
    fn production_template_enables_tls_auth_audit() {
        let t = ConfigTemplate::production_config_template();
        assert!(t.contains("log_level = \"warn\""));
        assert!(t.contains("enable_tls = true"));
        assert!(t.contains("enable_auth = true"));
        assert!(t.contains("enable_audit = true"));
        assert!(t.contains("enable_dashboard = false"));
    }

    #[test]
    fn home_network_template_enables_discovery_sections() {
        let t = ConfigTemplate::home_network_config_template();
        assert!(t.contains("enable_discovery = true"));
        assert!(t.contains("[discovery]"));
        assert!(t.contains("enable_multicast = true"));
    }

    #[test]
    fn dockerfile_template_exposes_base_port_and_multistage_build() {
        let cfg = CanonicalSongbirdConfig::default();
        let t = ConfigTemplate::dockerfile_template();
        assert!(t.contains("FROM rust:1.75-slim as builder"));
        assert!(t.contains(&format!("EXPOSE {}", cfg.network.base_port)));
        assert!(t.contains("CMD [\"songbird\", \"start\"]"));
    }
}
