// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use anyhow::{Context, Result};
use std::process::{Command, Stdio};
use tracing::debug;

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
    let env_string = {
        use std::fmt::Write;
        let mut s = String::new();
        for (i, (k, v)) in env_vars.iter().enumerate() {
            if i > 0 {
                s.push(' ');
            }
            let _ = write!(s, "{k}=\"{v}\"");
        }
        s
    };

    let command = format!("nohup {env_string} {remote_path} > /tmp/service.log 2>&1 &");

    ssh_exec(remote_host, &command, ssh_user, ssh_key)?;

    Ok(())
}
