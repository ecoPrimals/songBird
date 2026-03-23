// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Thin entry point for the `songbird-compute-bridge` binary.
//! The `UniBin` `songbird compute-bridge` delegates to [`songbird_compute_bridge::run`].

use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = songbird_compute_bridge::Args::parse();
    songbird_compute_bridge::run(args).await
}
