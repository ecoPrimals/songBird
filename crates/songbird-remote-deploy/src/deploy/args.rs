// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use clap::{Parser, Subcommand};
use songbird_types::constants::LOCALHOST;
use songbird_types::defaults::ports::DEFAULT_HTTP_PORT;

pub(super) fn default_federation_endpoint() -> String {
    format!("http://{LOCALHOST}:{DEFAULT_HTTP_PORT}")
}

/// CLI arguments for `songbird-deploy` (SSH, HTTP, list, and status subcommands).
#[derive(Parser, Debug)]
#[command(name = "songbird-deploy")]
#[command(about = "Agnostic remote service deployment for Songbird federation")]
pub struct Args {
    #[command(subcommand)]
    pub(super) command: Commands,

    /// Base URL of the Songbird federation API (discovery and coordination).
    ///
    /// Resolved from `--songbird-endpoint`, then `SONGBIRD_FEDERATION_ENDPOINT`, then
    /// the default `http://127.0.0.1:8080` (see module docs).
    #[arg(long, env = "SONGBIRD_FEDERATION_ENDPOINT", default_value_t = default_federation_endpoint())]
    pub(super) songbird_endpoint: String,
}

#[cfg(test)]
impl Args {
    pub(super) fn command_ref(&self) -> &Commands {
        &self.command
    }
}

#[derive(Subcommand, Debug)]
pub(super) enum Commands {
    /// Deploy a service to a remote tower via SSH
    Deploy {
        /// Target tower ID or address
        #[arg(long)]
        tower: String,

        /// Binary path to deploy
        #[arg(long)]
        binary: String,

        /// Remote destination path
        #[arg(long, default_value = "/tmp/deployed-service")]
        remote_path: String,

        /// Environment variables (can be specified multiple times)
        #[arg(long = "env", value_parser = parse_env_var)]
        env_vars: Vec<(String, String)>,

        /// SSH user (defaults to $USER)
        #[arg(long, env = "SSH_USER")]
        ssh_user: Option<String>,

        /// SSH key path
        #[arg(long, env = "SSH_KEY")]
        ssh_key: Option<String>,

        /// Auto-start after deployment
        #[arg(long, default_value = "true")]
        auto_start: bool,
    },

    /// Deploy a service via HTTP deployment API (adaptive)
    DeployHttp {
        /// Target tower HTTP endpoint (e.g. <http://192.0.2.10:8080>)
        #[arg(long)]
        tower: String,

        /// Binary path to deploy
        #[arg(long)]
        binary: String,

        /// Service name
        #[arg(long)]
        service: String,

        /// Environment variables (can be specified multiple times)
        #[arg(long = "env", value_parser = parse_env_var)]
        env_vars: Vec<(String, String)>,
    },

    /// List available towers from federation
    List {
        /// Show detailed information
        #[arg(long)]
        detailed: bool,
    },

    /// Check status of deployed services
    Status {
        /// Target tower ID or address
        #[arg(long)]
        tower: String,

        /// Service port to check
        #[arg(long)]
        port: Option<u16>,
    },
}

pub(super) fn parse_env_var(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid env var format: {s}. Expected KEY=VALUE"));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}
