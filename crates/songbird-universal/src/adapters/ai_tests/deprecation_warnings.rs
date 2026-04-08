// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::super::*;
use crate::adapters::discovery_test_sync::lock_discovery_env;
use songbird_config::capability_endpoints::CapabilityEndpointResolver;
use songbird_types::SongbirdResult;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tracing_subscriber::layer::SubscriberExt;

#[tokio::test]
async fn squirrel_endpoint_logs_deprecation_warning() -> SongbirdResult<()> {
    #[derive(Clone)]
    struct BufWriter(Arc<Mutex<Vec<u8>>>);
    impl Write for BufWriter {
        fn write(&mut self, d: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner).extend_from_slice(d);
            Ok(d.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let _g = lock_discovery_env();
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("CAPABILITY_AI_ENDPOINT");
    for key in ["AI_ENDPOINT", "AI_PROVIDER_ENDPOINT"] {
        songbird_process_env::remove_var(key);
    }
    songbird_process_env::set_var("SQUIRREL_ENDPOINT", "http://from-squirrel-warn:7722");

    let log_buf = Arc::new(Mutex::new(Vec::<u8>::new()));
    let w = Arc::clone(&log_buf);
    let subscriber = tracing_subscriber::registry().with(
        tracing_subscriber::fmt::layer()
            .without_time()
            .with_target(false)
            .with_level(false)
            .with_ansi(false)
            .with_writer(move || BufWriter(Arc::clone(&w))),
    );
    let _trace_guard = tracing::subscriber::set_default(subscriber);

    let adapter =
        AIAdapter::from_discovery_with_resolver(CapabilityEndpointResolver::new()).await?;
    assert_eq!(adapter.endpoint(), "http://from-squirrel-warn:7722");
    drop(_trace_guard);
    let logs = String::from_utf8_lossy(
        &log_buf.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone(),
    )
    .into_owned();
    assert!(logs.contains("SQUIRREL_ENDPOINT") && logs.contains("deprecated"), "logs were: {logs}");

    songbird_process_env::reset_overlay();
    Ok(())
}
