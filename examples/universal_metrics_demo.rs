use songbird_types::config::consolidated_canonical::CanonicalSongbirdConfig;
/// Universal Metrics Collection Demo
///
/// HARD MIGRATION - NO LEGACY COMPATIBILITY
/// 
/// This example demonstrates the new universal metrics collection system that:
/// - Automatically discovers ecosystem primals (../beardog, ../toadstool, ../nestgate)
/// - Collects metrics based on capabilities, not hardcoded names
/// - Works with any primal that provides metrics capabilities

// use songbird_universal::  // TEMPORARILY DISABLED - {
    CapabilityType, EcosystemPrimalDiscovery, EcosystemDiscoveryConfig,
    init_with_ecosystem_discovery,
};
use std::collections::HashMap;
use tracing::{error, info, warn};

/// Universal metrics collection result
#[derive(Debug)]
pub struct UniversalMetrics {
    pub primal_id: String,
    pub primal_name: String,
    pub capabilities: Vec<String>,
    pub metrics: HashMap<String, serde_json::Value>,
    pub health_status: String,
}

/// Universal metrics collector
pub struct UniversalMetricsCollector {
    discovery: EcosystemPrimalDiscovery,
}

impl UniversalMetricsCollector {
    /// Create new universal metrics collector
    ///
    /// # Errors
    /// Returns error if initialization fails
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let discovery = init_with_ecosystem_discovery().await?;
        Ok(Self { discovery })
    }

    /// Collect metrics from all discovered primals with metrics capabilities
    pub async fn collect_all_metrics(&self) -> Vec<UniversalMetrics> {
        let mut all_metrics = Vec::new();

        info!("🔍 Collecting metrics from all discovered primals...");

        for primal in self.discovery.get_discovered_primals().values() {
            info!("📊 Collecting metrics from: {}", primal.display_name);

            match self.collect_primal_metrics(primal).await {
                Ok(metrics) => {
                    info!("✅ Metrics collected from {}: {} data points", 
                          primal.display_name, metrics.metrics.len());
                    all_metrics.push(metrics);
                }
                Err(e) => {
                    warn!("⚠️  Failed to collect metrics from {}: {}", primal.display_name, e);
                    info!("   → This is expected if {} is not running", primal.display_name);
                }
            }
        }

        info!("🎯 Metrics collection complete: {} primals responded", all_metrics.len());
        all_metrics
    }

    /// Collect metrics from primals with specific capability
    pub async fn collect_metrics_by_capability(&self) -> Vec<UniversalMetrics> {
        let mut metrics = Vec::new();

        info!("🎯 Collecting metrics from primals with '{}' capability", capability);

        let capable_primals = self.discovery.find_by_custom_capability(capability);
        
        if capable_primals.is_empty() {
            warn!("❌ No primals found with '{}' capability", capability);
            return metrics;
        }

        for primal in capable_primals {
            info!("📊 Collecting {} metrics from: {}", capability, primal.display_name);

            match self.collect_primal_metrics(primal).await {
                Ok(primal_metrics) => {
                    info!("✅ {} metrics collected from {}", 
                          capability, primal.display_name);
                    metrics.push(primal_metrics);
                }
                Err(e) => {
                    warn!("⚠️  Failed to collect {} metrics from {}: {}", 
                          capability, primal.display_name, e);
                }
            }
        }

        metrics
    }

    /// Collect metrics from a specific primal
    fn collect_primal_metrics(Result<UniversalMetrics, Box<dyn std::error::Error>>) ->  {
        let client = reqwest::Client::new();
        let metrics_url = format!("{}/metrics", primal.api_endpoint);

        let response = client.get(&metrics_url).send().await?;
        
        if !response.status().is_success() {
            return Err(format!("Metrics endpoint returned status: {}", response.status()).into());
        }

        let metrics_data: HashMap<String, serde_json::Value> = response.json().await?;

        Ok(UniversalMetrics {
            primal_id: primal.id.clone(),
            primal_name: primal.display_name.clone(),
            capabilities: primal.capabilities.iter().map(|c| format!("{:?}", c)).collect(),
            metrics: metrics_data,
            health_status: format!("{:?}", primal.health),
        })
    }

    /// Get available capability types in the ecosystem
    pub fn get_available_capabilities(&self) -> Vec<String> {
        let mut capabilities = std::collections::HashSet::new();
        
        for primal in self.discovery.get_discovered_primals().values() {
            for cap in &primal.capabilities {
                capabilities.insert(format!("{:?}", cap));
            }
        }
        
        capabilities.into_iter().collect()
    }

    /// Display ecosystem overview
    pub fn display_ecosystem_overview(&self) {
        let primals = self.discovery.get_discovered_primals();
        
        info!("🌌 ECOSYSTEM OVERVIEW");
        info!("===================");
        info!("Total Primals Discovered: {}", primals.len());
        
        for primal in primals.values() {
            info!("🔹 {}", primal.display_name);
            info!("   ID: {}", primal.id);
            info!("   Health: {:?}", primal.health);
            info!("   API: {}", primal.api_endpoint);
            info!("   Capabilities: {}", primal.capabilities.len());
            for cap in &primal.capabilities {
                info!("     • {:?}", cap);
            }
            info!("");
        }
    }
}

#[tokio::main]
fn main(Result<(), Box<dyn std::error::Error>>) ->  {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Universal Metrics Collection Demo - HARD MIGRATION VERSION");
    info!("============================================================");

    // Create universal metrics collector
    let collector = UniversalMetricsCollector::new()?;

    // Display ecosystem overview
    collector.display_ecosystem_overview();

    // Collect metrics from all discovered primals
    info!("\n🎯 COLLECTING METRICS FROM ALL PRIMALS");
    let all_metrics = collector.collect_all_metrics().await;

    // Display collected metrics
    for metrics in &all_metrics {
        info!("\n📊 METRICS FROM: {}", metrics.primal_name);
        info!("   Health: {}", metrics.health_status);
        info!("   Capabilities: {:?}", metrics.capabilities);
        info!("   Metrics: {} data points", metrics.metrics.len());
        
        // Display key metrics (limit to avoid spam)
        let mut count = 0;
        for (key, value) in &metrics.metrics {
            if count < 5 {  // Show first 5 metrics
                info!("     • {}: {}", key, value);
                count += 1;
            } else {
                info!("     • ... and {} more metrics", metrics.metrics.len() - 5);
                break;
            }
        }
    }

    // Demonstrate capability-based metrics collection
    info!("\n🎯 CAPABILITY-BASED METRICS COLLECTION");
    let available_capabilities = collector.get_available_capabilities();
    info!("Available capabilities: {:?}", available_capabilities);

    // Collect security metrics (from any primal with security capability)
    let security_metrics = collector.collect_metrics_by_capability("encryption").await;
    info!("🔐 Security metrics collected from {} primals", security_metrics.len());

    // Collect compute metrics (from any primal with compute capability)
    let compute_metrics = collector.collect_metrics_by_capability("container_runtime").await;
    info!("🖥️  Compute metrics collected from {} primals", compute_metrics.len());

    // Collect storage metrics (from any primal with storage capability)
    let storage_metrics = collector.collect_metrics_by_capability("file_system").await;
    info!("💾 Storage metrics collected from {} primals", storage_metrics.len());

    info!("\n✅ UNIVERSAL METRICS DEMO COMPLETE");
    info!("Key Benefits:");
    info!("  • ✅ No hardcoded primal names");
    info!("  • ✅ Automatic ecosystem discovery");
    info!("  • ✅ Capability-based routing");
    info!("  • ✅ Works with any primal providing metrics");
    info!("  • ✅ Future-proof architecture");

    if all_metrics.is_empty() {
        info!("\n💡 TO SEE LIVE METRICS:");
        info!("   Start any primal in the ecosystem:");
        info!("   • cd ../beardog && cargo run");
        info!("   • cd ../toadstool && cargo run");
        info!("   • cd ../nestgate && cargo run");
        info!("   Then run this demo again!");
    }

    Ok(())
} 