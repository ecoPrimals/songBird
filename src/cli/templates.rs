//! Configuration templates for various deployment scenarios

/// Configuration template generator
pub struct ConfigTemplate;

impl ConfigTemplate {
    /// Generate a basic service template
    pub fn service_template() -> String {
        let env_config = crate::config::environment::EnvironmentConfig::default();
        format!(
            "[orchestrator]\nbind_address = \"{}\"\nport = {}\nlog_level = \"info\"\n\n\
            [network]\nenable_tls = false\nenable_http2 = true\n\n\
            [security]\nenable_auth = false\nenable_audit = false\n\n\
            [observability]\nenable_dashboard = true\nmetrics_interval_secs = 30\n",
            env_config.bind_address, env_config.bind_port
        )
    }

    /// Generate a development configuration
    pub fn development_config_template() -> String {
        let env_config = crate::config::environment::EnvironmentConfig::default();
        format!(
            "[orchestrator]\nbind_address = \"{}\"\nport = {}\nlog_level = \"debug\"\nenable_metrics = true\n\n\
            [network]\nenable_tls = false\nenable_http2 = true\n\n\
            [security]\nenable_auth = false\nenable_audit = false\n\n\
            [observability]\nenable_dashboard = true\nmetrics_interval_secs = 10\n",
            env_config.bind_address, env_config.bind_port
        )
    }

    /// Generate a production configuration
    pub fn production_config_template() -> String {
        let env_config = crate::config::environment::EnvironmentConfig::default();
        format!(
            "[orchestrator]\nbind_address = \"{}\"\nport = {}\nlog_level = \"warn\"\nenable_metrics = true\n\n\
            [network]\nenable_tls = true\nenable_http2 = true\n\n\
            [security]\nenable_auth = true\nenable_audit = true\n\n\
            [observability]\nenable_dashboard = false\nmetrics_interval_secs = 60\n",
            env_config.bind_address, env_config.bind_port
        )
    }

    /// Generate a home network configuration
    pub fn home_network_config_template() -> String {
        let env_config = crate::config::environment::EnvironmentConfig::default();
        format!(
            "[orchestrator]\nbind_address = \"{}\"\nport = {}\nlog_level = \"info\"\n\n\
            [network]\nenable_discovery = true\nenable_tls = false\n\n\
            [security]\nenable_basic_security = true\n\n\
            [discovery]\nenable_multicast = true\ndiscovery_interval_secs = 30\n",
            env_config.bind_address, env_config.bind_port
        )
    }

    /// Generate a simple Dockerfile template
    pub fn dockerfile_template() -> String {
        let env_config = crate::config::environment::EnvironmentConfig::default();
        format!(
            "FROM rust:1.75-slim as builder\n\n\
            RUN apt-get update && apt-get install -y pkg-config libssl-dev\n\n\
            WORKDIR /app\n\n\
            COPY . .\nRUN cargo build --release\n\n\
            FROM debian:bookworm-slim\n\n\
            RUN apt-get update && apt-get install -y ca-certificates curl\n\n\
            COPY --from=builder /app/target/release/songbird /usr/local/bin/songbird\n\n\
            EXPOSE {}\n\n\
            CMD [\"songbird\", \"start\"]\n",
            env_config.bind_port
        )
    }
}
