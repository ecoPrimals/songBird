// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Service discovery CLI command — wraps `songbird_discovery` for terminal output.

#![allow(missing_docs, reason = "thin command wrapper; behavior described in module docs")]

use crate::errors::SongbirdResult;
use songbird_discovery::traits::ServiceQuery;
use songbird_discovery::UniversalDiscoveryFactory;

pub async fn execute_discovery(
    timeout: u64,
    protocol: Option<String>,
    continuous: bool,
) -> SongbirdResult<()> {
    use songbird_discovery::ServiceDiscovery;

    println!("Discovering services...");

    if let Some(ref proto) = protocol {
        println!("  Filtering by protocol: {proto}");
    }

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout);

    loop {
        let adapter = match UniversalDiscoveryFactory::create_auto_detect().await {
            Ok(a) => a,
            Err(e) => {
                eprintln!("  Discovery initialization failed: {e}");
                return Ok(());
            }
        };

        let query = ServiceQuery {
            name: None,
            service_id: None,
            service_type: protocol.clone(),
            version: None,
            tags: Vec::new(),
            metadata: std::collections::HashMap::new(),
            health_status: None,
            limit: None,
            sort_by: None,
        };

        match adapter.discover(query).await {
            Ok(services) if services.is_empty() => {
                println!("  No services found.");
            }
            Ok(services) => {
                println!("  Found {} service(s):", services.len());
                for svc in &services {
                    println!(
                        "    {} v{} [{}] ({}:{})",
                        svc.name, svc.version, svc.service_type, svc.host, svc.port
                    );
                }
            }
            Err(e) => {
                eprintln!("  Discovery error: {e}");
            }
        }

        if !continuous || std::time::Instant::now() >= deadline {
            break;
        }

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    println!("Discovery complete.");
    Ok(())
}
