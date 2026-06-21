// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! # 🚀 Songbird Remote Deploy (Agnostic)
//!
//! **Philosophy:** Deploy ANY service to ANY tower via SSH
//!
//! This tool enables Songbird to orchestrate service deployment across
//! federation towers without hardcoding service names or configurations.
//!
//! ## Features
//! - ✅ **Agnostic:** Deploy any binary/service
//! - ✅ **SSH-based:** Secure, standard protocol
//! - ✅ **Environment-driven:** All config via env vars or CLI
//! - ✅ **No hardcoding:** Works with any service
//! - ✅ **Federation-aware:** Queries Songbird for tower info
//!
//! ## Federation endpoint
//!
//! The base URL for federation discovery defaults to the local development
//! endpoint built from `songbird_types::constants`. Override with
//! `SONGBIRD_FEDERATION_ENDPOINT` or `--songbird-endpoint`.
//!
//! ## Usage
//! ```bash
//! # Deploy compute bridge to Tower B
//! songbird-deploy \
//!   --tower tower-b-strandgate \
//!   --binary ./target/release/songbird-compute-bridge \
//!   --env COMPUTE_SERVICE_NAME="Tower B Compute" \
//!   --env COMPUTE_HOST=192.168.1.134 \
//!   --env COMPUTE_PORT=9000 \
//!   --env SONGBIRD_FEDERATION_ENDPOINT=http://192.0.2.10:8080
//! ```

mod args;
mod remote;
mod ssh;
mod tower;

pub use args::Args;

use std::collections::HashMap;

use anyhow::Result;
use tracing::info;

use args::Commands;
use remote::{DeploymentConfig, deploy_service};
use tower::{check_status, list_towers};

fn init_tracing() {
    let filter = songbird_process_env::var("RUST_LOG")
        .unwrap_or_else(|_| String::from("info,songbird_remote_deploy=debug"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

/// Run remote deploy CLI logic (standalone `songbird-deploy` or `songbird deploy`).
///
/// # Errors
///
/// Returns an error if CLI operations fail, including deployment, tower discovery, or HTTP/SSH I/O.
pub async fn run(args: Args) -> Result<()> {
    init_tracing();

    match args.command {
        Commands::Deploy {
            tower,
            binary,
            remote_path,
            env_vars,
            ssh_user,
            ssh_key,
            auto_start,
        } => {
            let effective_user = ssh_user
                .or_else(|| songbird_process_env::var("USER").ok())
                .unwrap_or_else(|| String::from("root"));
            deploy_service(DeploymentConfig {
                songbird_endpoint: &args.songbird_endpoint,
                tower_id: &tower,
                binary_path: &binary,
                remote_path: &remote_path,
                env_vars: &env_vars,
                ssh_user: &effective_user,
                ssh_key: ssh_key.as_deref(),
                auto_start,
            })
            .await?;
        }
        Commands::DeployHttp {
            tower,
            binary,
            service,
            env_vars,
        } => {
            let env_map: HashMap<String, String> = env_vars.into_iter().collect();
            let response =
                crate::http_deploy::deploy_via_http_adaptive(&tower, &binary, &service, env_map)
                    .await?;

            info!("✅ Deployment successful!");
            info!("   Deployment ID: {}", response.deployment_id);
            info!("   Status: {}", response.status);
            info!("   Message: {}", response.message);
            if let Some(url) = response.service_url {
                info!("   Service URL: {}", url);
            }
        }
        Commands::List {
            detailed,
        } => {
            list_towers(&args.songbird_endpoint, detailed).await?;
        }
        Commands::Status {
            tower,
            port,
        } => {
            check_status(&args.songbird_endpoint, &tower, port).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::args::{Args, Commands, default_federation_endpoint, parse_env_var};
    use super::tower::parse_tower_address;
    use clap::Parser;
    use songbird_types::constants::LOCALHOST;
    use songbird_types::defaults::ports::DEFAULT_HTTP_PORT;

    #[test]
    fn default_federation_endpoint_matches_types_constants() {
        let expected = format!("http://{LOCALHOST}:{DEFAULT_HTTP_PORT}");
        assert_eq!(
            default_federation_endpoint(),
            expected,
            "CLI default federation URL should match songbird_types::constants"
        );
    }

    #[test]
    fn parse_tower_address_strips_port() {
        assert_eq!(
            parse_tower_address("192.168.1.10:8443"),
            "192.168.1.10",
            "host:port should yield host only"
        );
    }

    #[test]
    fn parse_tower_address_host_without_port_unchanged() {
        assert_eq!(
            parse_tower_address("compute.local"),
            "compute.local",
            "address without ':' should be returned as-is"
        );
    }

    #[test]
    fn parse_tower_address_takes_first_colon_segment() {
        assert_eq!(
            parse_tower_address("fd00::1:8080"),
            "fd00",
            "current parser splits on first ':' (IPv6 not specially handled)"
        );
    }

    #[test]
    fn parse_env_var_accepts_key_value() {
        let (k, v) = parse_env_var("FOO=bar").unwrap();
        assert_eq!(k, "FOO");
        assert_eq!(v, "bar");
    }

    #[test]
    fn parse_env_var_accepts_empty_value() {
        let (k, v) = parse_env_var("EMPTY=").unwrap();
        assert_eq!(k, "EMPTY");
        assert_eq!(v, "");
    }

    #[test]
    fn parse_env_var_rejects_missing_equals() {
        let err = parse_env_var("noequals").unwrap_err();
        assert!(
            err.contains("Invalid env var format"),
            "error should mention invalid format: {err}"
        );
    }

    #[test]
    fn parse_env_var_splits_on_first_equals() {
        let (k, v) = parse_env_var("A=b=c").unwrap();
        assert_eq!(k, "A");
        assert_eq!(v, "b=c");
    }

    #[test]
    fn args_default_songbird_endpoint_uses_federation_default() {
        let args = Args::try_parse_from(["songbird-deploy", "list"])
            .expect("minimal list command should parse");
        assert_eq!(
            args.songbird_endpoint,
            default_federation_endpoint(),
            "omitted --songbird-endpoint should match default_federation_endpoint()"
        );
    }

    #[test]
    fn args_parses_deploy_subcommand() {
        let args = Args::try_parse_from([
            "songbird-deploy",
            "--songbird-endpoint",
            "http://example.test:8080",
            "deploy",
            "--tower",
            "tower-a",
            "--binary",
            "/usr/bin/true",
        ])
        .unwrap();
        assert_eq!(args.songbird_endpoint, "http://example.test:8080");
        match args.command_ref() {
            Commands::Deploy {
                tower,
                binary,
                remote_path,
                auto_start,
                ..
            } => {
                assert_eq!(tower, "tower-a");
                assert_eq!(binary, "/usr/bin/true");
                assert_eq!(remote_path, "/tmp/deployed-service");
                assert!(auto_start);
            }
            _ => panic!("expected Deploy variant"),
        }
    }

    #[test]
    fn args_parses_deploy_http_subcommand() {
        let args = Args::try_parse_from([
            "songbird-deploy",
            "deploy-http",
            "--tower",
            "http://127.0.0.1:9000",
            "--binary",
            "./app",
            "--service",
            "mysvc",
        ])
        .unwrap();
        match args.command_ref() {
            Commands::DeployHttp {
                tower,
                binary,
                service,
                env_vars,
            } => {
                assert_eq!(tower, "http://127.0.0.1:9000");
                assert_eq!(binary, "./app");
                assert_eq!(service, "mysvc");
                assert!(env_vars.is_empty());
            }
            _ => panic!("expected DeployHttp variant"),
        }
    }

    #[test]
    fn args_parses_list_subcommand() {
        let args = Args::try_parse_from(["songbird-deploy", "list", "--detailed"])
            .expect("list --detailed should parse");
        match args.command_ref() {
            Commands::List {
                detailed,
            } => assert!(detailed, "--detailed should set flag"),
            _ => panic!("expected List variant"),
        }
    }

    #[test]
    fn args_parses_status_with_port() {
        let args = Args::try_parse_from([
            "songbird-deploy",
            "status",
            "--tower",
            "tower-z",
            "--port",
            "9000",
        ])
        .expect("status with port should parse");
        match args.command_ref() {
            Commands::Status {
                tower,
                port,
            } => {
                assert_eq!(tower, "tower-z");
                assert_eq!(*port, Some(9000_u16));
            }
            _ => panic!("expected Status variant"),
        }
    }

    #[test]
    fn args_parses_status_without_port() {
        let args = Args::try_parse_from(["songbird-deploy", "status", "--tower", "tower-z"])
            .expect("status without port should parse");
        match args.command_ref() {
            Commands::Status {
                port,
                ..
            } => assert!(port.is_none()),
            _ => panic!("expected Status variant"),
        }
    }

    #[test]
    fn parse_tower_address_empty_string() {
        assert_eq!(parse_tower_address(""), "", "empty address should yield empty host");
    }

    #[test]
    fn parse_tower_address_leading_colon_is_empty_host() {
        assert_eq!(
            parse_tower_address(":22"),
            "",
            "':port' splits to empty first segment (documents current behavior)"
        );
    }

    #[test]
    fn parse_tower_address_preserves_ipv4_mapped_literal() {
        assert_eq!(
            parse_tower_address("127.0.0.1:65535"),
            "127.0.0.1",
            "IPv4 host:port should strip port"
        );
    }

    #[test]
    fn parse_env_var_accepts_leading_equals_in_value() {
        let (k, v) = parse_env_var("KEY==value").unwrap();
        assert_eq!(k, "KEY");
        assert_eq!(v, "=value", "only first '=' splits; remainder is value");
    }

    #[test]
    fn parse_env_var_rejects_empty_input() {
        let err = parse_env_var("").unwrap_err();
        assert!(err.contains("Invalid env var format"), "got {err}");
    }

    #[test]
    fn deploy_http_rejects_missing_required_flags() {
        let err = Args::try_parse_from(["songbird-deploy", "deploy-http", "--tower", "http://h/"]);
        assert!(err.is_err(), "deploy-http without --binary/--service should fail");
    }

    #[test]
    fn deploy_rejects_missing_tower() {
        let err = Args::try_parse_from(["songbird-deploy", "deploy", "--binary", "/bin/true"]);
        assert!(err.is_err(), "deploy without --tower should fail");
    }

    #[test]
    fn status_port_zero_is_valid_clap_edge() {
        let args =
            Args::try_parse_from(["songbird-deploy", "status", "--tower", "t", "--port", "0"])
                .expect("port 0 is a valid u16");
        match args.command_ref() {
            Commands::Status {
                port,
                ..
            } => assert_eq!(*port, Some(0)),
            _ => panic!("expected Status"),
        }
    }
}
