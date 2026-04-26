// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use anyhow::{Context, Result};
use std::process::{Command, Stdio};
use tracing::debug;

pub(super) fn format_ssh_env_exports(env_vars: &[(String, String)]) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    for (i, (k, v)) in env_vars.iter().enumerate() {
        if i > 0 {
            s.push(' ');
        }
        let _ = write!(s, "{k}=\"{v}\"");
    }
    s
}

/// Builds the remote shell command used by [`start_remote_service`] (pure; unit-tested).
pub(super) fn build_nohup_remote_launch_command(
    remote_path: &str,
    env_vars: &[(String, String)],
) -> String {
    let env_string = format_ssh_env_exports(env_vars);
    let log_path = format!("{remote_path}.log");
    format!("nohup {env_string} {remote_path} > {log_path} 2>&1 &")
}

pub(super) fn scp_copy(
    local_path: &str,
    remote_host: &str,
    remote_path: &str,
    ssh_user: &str,
    ssh_key: Option<&str>,
) -> Result<()> {
    let mut cmd = Command::new("scp");

    if let Some(key) = ssh_key {
        cmd.arg("-i").arg(key);
    }

    cmd.arg(local_path)
        .arg(format!("{ssh_user}@{remote_host}:{remote_path}"))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    debug!("Executing: {:?}", cmd);

    let status = cmd.status().context("Failed to execute scp")?;

    if !status.success() {
        anyhow::bail!("SCP failed with status: {status}");
    }

    Ok(())
}

pub(super) fn ssh_exec(
    remote_host: &str,
    command: &str,
    ssh_user: &str,
    ssh_key: Option<&str>,
) -> Result<()> {
    let mut cmd = Command::new("ssh");

    if let Some(key) = ssh_key {
        cmd.arg("-i").arg(key);
    }

    cmd.arg(format!("{ssh_user}@{remote_host}"))
        .arg(command)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    debug!("Executing: {:?}", cmd);

    let status = cmd.status().context("Failed to execute ssh")?;

    if !status.success() {
        anyhow::bail!("SSH command failed with status: {status}");
    }

    Ok(())
}

pub(super) fn start_remote_service(
    remote_host: &str,
    remote_path: &str,
    env_vars: &[(String, String)],
    ssh_user: &str,
    ssh_key: Option<&str>,
) -> Result<()> {
    let command = build_nohup_remote_launch_command(remote_path, env_vars);

    ssh_exec(remote_host, &command, ssh_user, ssh_key)?;

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::{build_nohup_remote_launch_command, format_ssh_env_exports};

    #[test]
    fn format_ssh_env_exports_joins_exports() {
        let env = vec![("A".into(), "1".into()), ("B".into(), "two".into())];
        assert_eq!(format_ssh_env_exports(&env), "A=\"1\" B=\"two\"");
    }

    #[test]
    fn format_ssh_env_exports_empty() {
        assert_eq!(format_ssh_env_exports(&[]), "");
    }

    #[test]
    fn build_nohup_remote_launch_command_includes_path_and_log() {
        let env = vec![("PORT".into(), "8080".into())];
        let cmd = build_nohup_remote_launch_command("/opt/app/bin", &env);
        assert_eq!(cmd, "nohup PORT=\"8080\" /opt/app/bin > /opt/app/bin.log 2>&1 &");
    }

    #[test]
    fn build_nohup_escapes_values_via_quoting() {
        let env = vec![("X".into(), "a b".into())];
        let cmd = build_nohup_remote_launch_command("/run/svc", &env);
        assert!(cmd.contains("X=\"a b\""), "command: {cmd}");
    }
}
