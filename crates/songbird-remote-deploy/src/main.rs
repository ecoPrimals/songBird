// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Thin entry point for the `songbird-deploy` binary.
//! The `UniBin` `songbird deploy` delegates to [`songbird_remote_deploy::run`].

use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = songbird_remote_deploy::Args::parse();
    songbird_remote_deploy::run(args).await
}
