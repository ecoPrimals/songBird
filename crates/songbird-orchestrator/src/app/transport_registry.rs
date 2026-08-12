// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Unified transport lifecycle registry — vertebrate evolution (Wave 157a/d).
//!
//! Holds all registered transports via a type-erased adapter, providing uniform
//! lifecycle management: start, health aggregation, and graceful shutdown across
//! all 9+ transport layers.
//!
//! Uses a `TransportEntry` boxed-closure adapter to achieve dyn-compatibility
//! without requiring `async_trait` or unstable features.

use songbird_types::{SongbirdResult, TransportHealth};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Type-erased transport entry for dyn-compatible async lifecycle.
pub struct TransportEntry {
    name: &'static str,
    is_ready: Box<dyn Fn() -> BoxFut<'static, bool> + Send + Sync>,
    start: Box<dyn Fn() -> BoxFut<'static, SongbirdResult<()>> + Send + Sync>,
    shutdown: Box<dyn Fn() -> BoxFut<'static, SongbirdResult<()>> + Send + Sync>,
    health: Box<dyn Fn() -> BoxFut<'static, TransportHealth> + Send + Sync>,
}

/// Register a `CanonicalTransport` implementor as a type-erased entry.
///
/// The transport must be `'static + Send + Sync`.
pub fn entry_from<T>(transport: Arc<T>) -> TransportEntry
where
    T: songbird_types::CanonicalTransport + 'static,
{
    let name = transport.transport_name();

    let t1 = Arc::clone(&transport);
    let t2 = Arc::clone(&transport);
    let t3 = Arc::clone(&transport);
    let t4 = Arc::clone(&transport);

    TransportEntry {
        name,
        is_ready: Box::new(move || {
            let t = Arc::clone(&t1);
            Box::pin(async move { t.is_ready().await })
        }),
        shutdown: Box::new(move || {
            let t = Arc::clone(&t2);
            Box::pin(async move { t.shutdown().await })
        }),
        start: Box::new(move || {
            let t = Arc::clone(&t3);
            Box::pin(async move { t.start().await })
        }),
        health: Box::new(move || {
            let t = Arc::clone(&t4);
            Box::pin(async move { t.health().await })
        }),
    }
}

/// Unified registry for transport lifecycle management.
///
/// The orchestrator registers all active transports at boot, then uses this
/// registry for health aggregation (Stage 6) and graceful shutdown.
pub struct TransportRegistry {
    transports: RwLock<Vec<TransportEntry>>,
}

impl TransportRegistry {
    /// Create an empty transport registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            transports: RwLock::new(Vec::new()),
        }
    }

    /// Register a transport for lifecycle management.
    pub async fn register(&self, entry: TransportEntry) {
        debug!(transport = entry.name, "registered transport in lifecycle registry");
        self.transports.write().await.push(entry);
    }

    /// Start all registered transports.
    ///
    /// Transports that fail to start are logged but do not block others.
    pub async fn start_all(&self) -> SongbirdResult<()> {
        let transports = self.transports.read().await;
        info!(count = transports.len(), "starting all registered transports");
        for entry in transports.iter() {
            match (entry.start)().await {
                Ok(()) => debug!(transport = entry.name, "transport started"),
                Err(e) => {
                    warn!(transport = entry.name, error = %e, "transport failed to start (non-fatal)");
                }
            }
        }
        Ok(())
    }

    /// Gracefully shut down all registered transports.
    pub async fn shutdown_all(&self) {
        let transports = self.transports.read().await;
        info!(count = transports.len(), "shutting down all registered transports");
        for entry in transports.iter() {
            match (entry.shutdown)().await {
                Ok(()) => debug!(transport = entry.name, "transport shut down"),
                Err(e) => error!(transport = entry.name, error = %e, "transport shutdown failed"),
            }
        }
    }

    /// Aggregate health from all registered transports.
    pub async fn health_all(&self) -> Vec<(&'static str, TransportHealth)> {
        let transports = self.transports.read().await;
        let mut results = Vec::with_capacity(transports.len());
        for entry in transports.iter() {
            let health = (entry.health)().await;
            results.push((entry.name, health));
        }
        results
    }

    /// Number of registered transports.
    pub async fn count(&self) -> usize {
        self.transports.read().await.len()
    }

    /// Number of transports currently reporting ready.
    pub async fn ready_count(&self) -> usize {
        let transports = self.transports.read().await;
        let mut ready = 0;
        for entry in transports.iter() {
            if (entry.is_ready)().await {
                ready += 1;
            }
        }
        ready
    }
}

impl Default for TransportRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;
    use songbird_types::{CanonicalTransport, SongbirdResult, TransportEndpoint, TransportHealth};

    struct MockTransport {
        name: &'static str,
    }

    impl CanonicalTransport for MockTransport {
        fn transport_name(&self) -> &'static str {
            self.name
        }

        async fn is_ready(&self) -> bool {
            true
        }

        async fn start(&self) -> SongbirdResult<()> {
            Ok(())
        }

        async fn shutdown(&self) -> SongbirdResult<()> {
            Ok(())
        }

        async fn health(&self) -> TransportHealth {
            TransportHealth {
                ready: true,
                active_connections: 42,
                message: format!("{} healthy", self.name),
            }
        }

        fn endpoints(&self) -> Vec<TransportEndpoint> {
            vec![]
        }
    }

    #[tokio::test]
    async fn registry_lifecycle() {
        let registry = TransportRegistry::new();
        assert_eq!(registry.count().await, 0);

        registry
            .register(entry_from(Arc::new(MockTransport {
                name: "TestA",
            })))
            .await;
        registry
            .register(entry_from(Arc::new(MockTransport {
                name: "TestB",
            })))
            .await;

        assert_eq!(registry.count().await, 2);
        assert_eq!(registry.ready_count().await, 2);

        registry.start_all().await.unwrap();

        let health = registry.health_all().await;
        assert_eq!(health.len(), 2);
        assert_eq!(health[0].0, "TestA");
        assert!(health[0].1.ready);
        assert_eq!(health[0].1.active_connections, 42);

        registry.shutdown_all().await;
    }

    #[tokio::test]
    async fn empty_registry_is_safe() {
        let registry = TransportRegistry::new();
        registry.start_all().await.unwrap();
        registry.shutdown_all().await;
        let health = registry.health_all().await;
        assert!(health.is_empty());
    }
}
