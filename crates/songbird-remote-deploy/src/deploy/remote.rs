// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use anyhow::Result;
use tracing::{info, warn};

use super::ssh::{scp_copy, ssh_exec, start_remote_service};
use super::tower::{get_tower_info, parse_tower_address, verify_service_health};

pub(super) struct DeploymentConfig<'a> {
    pub(super) songbird_endpoint: &'a str,
    pub(super) tower_id: &'a str,
    pub(super) binary_path: &'a str,
    pub(super) remote_path: &'a str,
    pub(super) env_vars: &'a [(String, String)],
    pub(super) ssh_user: &'a str,
    pub(super) ssh_key: Option<&'a str>,
    pub(super) auto_start: bool,
}

pub(super) async fn deploy_service(config: DeploymentConfig<'_>) -> Result<()> {
    info!("🚀 Deploying service to tower: {}", config.tower_id);

    let tower_info = get_tower_info(config.songbird_endpoint, config.tower_id).await?;
    let tower_address = parse_tower_address(&tower_info.node_address);

    info!("📡 Target: {} ({})", tower_info.node_name, tower_address);
    info!("📦 Binary: {}", config.binary_path);
    info!("📍 Remote path: {}", config.remote_path);

    info!("📤 Copying binary...");
    scp_copy(
        config.binary_path,
        &tower_address,
        config.remote_path,
        config.ssh_user,
        config.ssh_key,
    )?;
    info!("✅ Binary copied successfully");

    info!("🔧 Making binary executable...");
    ssh_exec(
        &tower_address,
        &format!("chmod +x {}", config.remote_path),
        config.ssh_user,
        config.ssh_key,
    )?;
    info!("✅ Binary is executable");

    if config.auto_start {
        info!("🎬 Starting service...");
        start_remote_service(
            &tower_address,
            config.remote_path,
            config.env_vars,
            config.ssh_user,
            config.ssh_key,
        )?;
        info!("✅ Service started");

        if let Some(port) = config
            .env_vars
            .iter()
            .find(|(k, _)| k.ends_with("PORT"))
            .and_then(|(_, v)| v.parse::<u16>().ok())
        {
            info!("🔍 Verifying service health...");
            if let Err(e) = verify_service_health(&tower_address, port).await {
                warn!("⚠️  Service may not be healthy yet: {}", e);
            } else {
                info!("✅ Service is healthy!");
            }
        }
    }

    info!("");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("🎉 Deployment Complete!");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");
    info!("Tower: {} ({})", tower_info.node_name, tower_address);
    info!("Service: {}", config.remote_path);
    info!(
        "Status: {}",
        if config.auto_start {
            "Running"
        } else {
            "Deployed (not started)"
        }
    );

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::DeploymentConfig;

    #[test]
    fn deployment_config_fields_for_auto_start_and_port_detection() {
        let env = vec![("APP_PORT".into(), "9000".into()), ("OTHER".into(), "x".into())];
        let cfg = DeploymentConfig {
            songbird_endpoint: "http://127.0.0.1:8080",
            tower_id: "tower-a",
            binary_path: "./target/release/app",
            remote_path: "/tmp/app",
            env_vars: &env,
            ssh_user: "deploy",
            ssh_key: None,
            auto_start: false,
        };
        assert_eq!(cfg.tower_id, "tower-a");
        assert!(!cfg.auto_start);
        assert!(cfg.ssh_key.is_none());
        let port = cfg
            .env_vars
            .iter()
            .find(|(k, _)| k.ends_with("PORT"))
            .and_then(|(_, v)| v.parse::<u16>().ok());
        assert_eq!(port, Some(9000));
    }
}
